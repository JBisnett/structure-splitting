use syntax;
use rustc::ty::TyCtxt;
use rustc::ty;
use rustc::mir;
use rustc::middle;
use rustc::mir::visit;
use rustc::mir::visit::Visitor;
// use rustc_data_structures::indexed_vec::Idx;
use syntax::codemap::DUMMY_SP;

use split_struct::LocalMap;

pub fn split_function_call<'a, 'tcx>(tcx: TyCtxt<'a, 'tcx, 'tcx>,
                                     mir: &'a mut mir::Mir<'tcx>,
                                     decl_maps: &'a LocalMap<'tcx>) {
  let immut_mir = mir.clone();
  let mut splitter = FunctionCallSplitter::new(tcx, mir, decl_maps);
  splitter.visit_mir(&immut_mir);
}

#[derive(new)]
struct FunctionCallSplitter<'a, 'tcx: 'a> {
  tcx: TyCtxt<'a, 'tcx, 'tcx>,
  mir: &'a mut mir::Mir<'tcx>,
  decl_maps: &'a LocalMap<'tcx>,
}

impl<'a, 'tcx> visit::Visitor<'tcx> for FunctionCallSplitter<'a, 'tcx> {
  fn visit_terminator_kind(&mut self,
                           block: mir::BasicBlock,
                           terminator: &mir::TerminatorKind<'tcx>,
                           _: mir::Location) {

    let source_info = mir::SourceInfo {
      span: syntax::codemap::DUMMY_SP,
      scope: mir::ARGUMENT_VISIBILITY_SCOPE,
    };
    if let &mir::TerminatorKind::Call { ref args, 
        //ref destination, 
            .. } =
      terminator {
      let index_ty = self.tcx.types.u128;
      let tuple_assignment_lvalues = |tup: mir::Local| {
        let tup_local = tup.clone();
        move |(index, arg_local)| {
          let op_lit = mir::Literal::Value{
                        value: middle::const_val::ConstVal::Integral
                            (middle::const_val::ConstInt::
                             U128(index as u128))};
          let op_index = mir::Operand::Constant(mir::Constant {
            span: DUMMY_SP,
            ty: index_ty.clone(),
            literal: op_lit,
          });
          let tuple_projection =
            mir::Lvalue::Projection(box mir::LvalueProjection {
              base: mir::Lvalue::Local(tup_local),
              elem: mir::ProjectionElem::Index(op_index),
            });
          let local_lvalue = mir::Lvalue::Local(arg_local);
          (tuple_projection, local_lvalue)
        }
      };
      let new_args = args.iter()
        .map(|arg| if let mir::Operand::Consume(mir::Lvalue::Local(local)) =
          *arg {
          if let Some(type_locals) = self.decl_maps.get(&local) {
            let (tup_ty_arr, local_vec): (Vec<ty::Ty>, Vec<mir::Local>) =
              type_locals.iter().unzip();
            let tup_ty = self.tcx.intern_tup(&*tup_ty_arr, false);
            let tup_local =
              self.mir.local_decls.push(mir::LocalDecl::new_temp(tup_ty));
            let assignments = local_vec.into_iter()
              .enumerate()
              .map(tuple_assignment_lvalues(tup_local))
              .map(|(tuple_projection, local_lvalue)| {
                let assign_kind =
                          mir::StatementKind::Assign
                          (tuple_projection, mir::Rvalue::Use
                           (mir::Operand::Consume(local_lvalue)));
                mir::Statement {
                  source_info: source_info,
                  kind: assign_kind,
                }
              });
            self.mir.basic_blocks_mut()[block].statements.extend(assignments);
            mir::Operand::Consume(mir::Lvalue::Local(local))
          } else {
            arg.clone()
          }
        } else {
          arg.clone()
        })
        .collect::<Vec<_>>();
      //let new_destination = if let &Some((ref lvalue, block)) = destination {};
      let ref mut current_block = self.mir.basic_blocks_mut()[block];
      if let Some(ref mut current_terminator) = current_block.terminator {
        if let mir::TerminatorKind::Call { ref mut args,
                                           //ref mut destination,
                                           .. } = current_terminator.kind {
          *args = new_args;
        }
      }
    }
  }
}
