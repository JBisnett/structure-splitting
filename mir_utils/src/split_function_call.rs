use syntax;
use rustc::ty::TyCtxt;
use rustc::mir;
use rustc::mir::visit;
use rustc_data_structures::indexed_vec::Idx;

use split_struct::{SplitMap, LocalMap};
use struct_base_replacer::StructFieldReplacer;
use split_utils;

#[derive(new)]
pub struct FunctionCallSplitter<'a, 'tcx: 'a + 'mr, 'mr> {
  tcx: TyCtxt<'a, 'tcx, 'tcx>,
  mir: &'mr mir::Mir<'tcx>,
  decl_maps: &'a LocalMap<'tcx>,
}

impl<'a, 'tcx, 'mr> visit::MutVisitor<'tcx>
  for FunctionCallSplitter<'a, 'tcx, 'mr> {
  fn visit_terminator_kind(&mut self,
                           block: mir::BasicBlock,
                           terminator: &mut mir::TerminatorKind<'tcx>,
                           location: mir::Location) {
    if let &mut mir::TerminatorKind::Call { ref func,
                                            ref mut args,
                                            ref mut destination,
                                            ref cleanup } = terminator {
      for arg in args {
        if let mir::Operand::Consume(mir::Lvalue::Local(local)) = *arg {
          if let Some(types) = self.decl_maps.get(&local) {
            // create tuple
          }
        }
      }
    };
  }
}
