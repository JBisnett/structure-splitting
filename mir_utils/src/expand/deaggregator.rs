use rustc::ty::TyCtxt;
use rustc::mir::*;
use rustc::mir::transform::{MirPass, MirSource, Pass};
use rustc_data_structures::indexed_vec::Idx;
use rustc::session;

pub struct Deaggregator;

impl Pass for Deaggregator {}

macro_rules! bug {
  () => ( bug!("impossible case reached") );
  ($($message:tt)*) => ({
    $crate::session::bug_fmt(file!(), line!(), format_args!($($message)*))
  })
}

macro_rules! span_bug {
  ($span:expr, $($message:tt)*) => ({
    session::span_bug_fmt(file!(), line!(), $span, format_args!($($message)*))
  })
}
impl<'tcx> MirPass<'tcx> for Deaggregator {
  fn run_pass<'a>(&mut self,
                  tcx: TyCtxt<'a, 'tcx, 'tcx>,
                  source: MirSource,
                  mir: &mut Mir<'tcx>) {
    // Do not trigger on constants.  Could be revised in future
    if let MirSource::Fn(_) = source {} else {
      return;
    }
    // In fact, we might not want to trigger in other cases.
    // Ex: when we could use SROA.  See issue #35259

    for bb in mir.basic_blocks_mut() {
      let mut curr: usize = 0;
      while let Some(idx) = get_aggregate_statement_index(curr,
                                                          &bb.statements) {
        // do the replacement
        let src_info = bb.statements[idx].source_info;
        let suffix_stmts = bb.statements.split_off(idx + 1);
        let orig_stmt = bb.statements.pop().unwrap();
        let (lhs, rhs) = match orig_stmt.kind {
          StatementKind::Assign(ref lhs, ref rhs) => (lhs, rhs),
          _ => span_bug!(src_info.span, "expected assign, not {:?}", orig_stmt),
        };
        let (agg_kind, operands) = match rhs {
          &Rvalue::Aggregate(ref agg_kind, ref operands) => {
            (agg_kind, operands)
          }
          _ => span_bug!(src_info.span, "expected aggregate, not {:?}", rhs),
        };
        let (adt_def, variant, substs) = match agg_kind {
          &AggregateKind::Adt(adt_def, variant, substs, None) => {
            (adt_def, variant, substs)
          }
          _ => span_bug!(src_info.span, "expected struct, not {:?}", rhs),
        };
        let n = bb.statements.len();
        bb.statements.reserve(n + operands.len() + suffix_stmts.len());
        for (i, op) in operands.iter().enumerate() {
          let ref variant_def = adt_def.variants[variant];
          let ty = variant_def.fields[i].ty(tcx, substs);
          let rhs = Rvalue::Use(op.clone());

          let lhs_cast = if adt_def.variants.len() > 1 {
            Lvalue::Projection(Box::new(LvalueProjection {
              base: lhs.clone(),
              elem: ProjectionElem::Downcast(adt_def, variant),
            }))
          } else {
            lhs.clone()
          };

          let lhs_proj = Lvalue::Projection(Box::new(LvalueProjection {
            base: lhs_cast,
            elem: ProjectionElem::Field(Field::new(i), ty),
          }));
          let new_statement = Statement {
            source_info: src_info,
            kind: StatementKind::Assign(lhs_proj, rhs),
          };
          bb.statements.push(new_statement);
        }

        // if the aggregate was an enum, we need to set the discriminant
        if adt_def.variants.len() > 1 {
          let set_discriminant = Statement {
            kind: StatementKind::SetDiscriminant {
              lvalue: lhs.clone(),
              variant_index: variant,
            },
            source_info: src_info,
          };
          bb.statements.push(set_discriminant);
        };

        curr = bb.statements.len();
        bb.statements.extend(suffix_stmts);
      }
    }
  }
}

fn get_aggregate_statement_index<'a, 'tcx, 'b>(start: usize,
                                         statements: &Vec<Statement<'tcx>>)
                                         -> Option<usize> {
  for i in start..statements.len() {
    let ref statement = statements[i];
    let rhs = match statement.kind {
      StatementKind::Assign(_, ref rhs) => rhs,
      _ => continue,
    };
    let (kind, operands) = match rhs {
      &Rvalue::Aggregate(ref kind, ref operands) => (kind, operands),
      _ => continue,
    };
    let (_, _) = match kind {
      &AggregateKind::Adt(adt_def, variant, _, None) => (adt_def, variant),
      _ => continue,
    };
    if operands.len() == 0 {
      // don't deaggregate ()
      continue;
    }
    return Some(i);
  }
  None
}
