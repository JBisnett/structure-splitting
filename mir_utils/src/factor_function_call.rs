use syntax;
use rustc::ty::TyCtxt;
use rustc::mir;
use rustc::mir::visit;
use rustc::mir::visit::Visitor;

// Factors projections out of function call terminators.
// TODO: Factor out statics

pub fn factor_mir<'a, 'tcx: 'a>(tcx: TyCtxt<'a, 'tcx, 'tcx>,
                                mir: &'a mut mir::Mir<'tcx>) {
  FunctionCallFactorer::new(tcx, mir).factor();
}

struct FunctionCallFactorer<'a, 'tcx: 'a> {
  tcx: TyCtxt<'a, 'tcx, 'tcx>,
  mir: &'a mut mir::Mir<'tcx>,
}

impl<'a, 'tcx: 'a> FunctionCallFactorer<'a, 'tcx> {
  fn new(tcx: TyCtxt<'a, 'tcx, 'tcx>, mir: &'a mut mir::Mir<'tcx>) -> Self {
    Self {
      tcx: tcx,
      mir: mir,
    }
  }
  fn factor(&mut self) {
    let immut_mir = self.mir.clone();
    self.visit_mir(&immut_mir);
    // This should not be accessible outside of this function
  }
}
impl<'a, 'tcx> visit::Visitor<'tcx> for FunctionCallFactorer<'a, 'tcx> {
  fn visit_terminator_kind(&mut self,
                           block: mir::BasicBlock,
                           terminator: &mir::TerminatorKind<'tcx>,
                           _: mir::Location) {
    let source_info = mir::SourceInfo {
      span: syntax::codemap::DUMMY_SP,
      scope: mir::ARGUMENT_VISIBILITY_SCOPE,
    };
    let ref mut true_mir = self.mir;
    if let &mir::TerminatorKind::Call { ref func,
                                        ref args,
                                        ref destination,
                                        ref cleanup } = terminator {
      let mut new_args = vec![];
      for arg in args {
        if let op @ mir::Operand::Consume(mir::Lvalue::Projection(..)) =
          arg.clone() {
          let local_ty = arg.ty(true_mir, self.tcx);
          let new_local = true_mir.local_decls
            .push(mir::LocalDecl::new_temp(local_ty));
          let new_lvalue = mir::Lvalue::Local(new_local);
          let assign_kind = mir::StatementKind::Assign(new_lvalue.clone(),
                                                       mir::Rvalue::Use(op));
          let new_assignment = mir::Statement {
            source_info: source_info,
            kind: assign_kind,
          };
          true_mir.basic_blocks_mut()[block]
            .statements
            .push(new_assignment);
          new_args.push(mir::Operand::Consume(new_lvalue));
        } else {
          new_args.push(arg.clone());
        }
      }
      let new_destination =
        if let &Some((ref proj @ mir::Lvalue::Projection(..), target)) =
          destination {
          let proj_ty = proj.ty(true_mir, self.tcx).to_ty(self.tcx);
          let new_local = true_mir.local_decls
            .push(mir::LocalDecl::new_temp(proj_ty));
          let new_lvalue = mir::Lvalue::Local(new_local);
          let assign_kind =
            mir::StatementKind::Assign(proj.clone(),
                                       mir::Rvalue::Use(
                                         mir::Operand::Consume(
                                           new_lvalue.clone())));
          let new_assignment = mir::Statement {
            source_info: source_info,
            kind: assign_kind,
          };
          let new_terminator = mir::Terminator {
            source_info: source_info,
            kind: mir::TerminatorKind::Goto { target: target },
          };
          let new_block_data = mir::BasicBlockData {
            statements: vec![new_assignment],
            terminator: Some(new_terminator),
            is_cleanup: false,
          };
          let new_block = true_mir.basic_blocks_mut().push(new_block_data);
          Some((new_lvalue, new_block))
        } else {
          destination.clone()
        };
      true_mir.basic_blocks_mut()[block].terminator = Some(mir::Terminator {
        source_info: source_info,
        kind: mir::TerminatorKind::Call {
          func: func.clone(),
          args: new_args,
          destination: new_destination,
          cleanup: cleanup.clone(),
        },
      })
    }
  }
}
