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
  ty2structsplit: &'a SplitMap<'tcx>,
  decl_maps: &'a LocalMap<'tcx>,
}

impl<'a, 'tcx, 'mr> visit::MutVisitor<'tcx>
  for FunctionCallSplitter<'a, 'tcx, 'mr> {
  fn visit_mir(&mut self, mir: &mut mir::Mir<'tcx>) {
    let source_info = mir::SourceInfo {
      span: syntax::codemap::DUMMY_SP,
      scope: mir::ARGUMENT_VISIBILITY_SCOPE,
    };
    let field_replacer = StructFieldReplacer::new(self.tcx,
                                                  self.mir,
                                                  self.ty2structsplit,
                                                  self.decl_maps);
    for (index, data) in mir.basic_blocks().clone().iter_enumerated_mut() {
      // Check each block terminator, if it is a function call, then
      // check if return or args are split structs
      // Need to handle nests, and struct fields. Should probably do a refactor
      // of code we ahve already
      {
        let mut terminator = data.terminator_mut();
        if let mir::TerminatorKind::Call { ref func,
                                           ref args,
                                           ref destination,
                                           cleanup } = terminator.kind {
          for operand in args.iter() {
            let ty = operand.ty(mir, self.tcx);
          }
        }
      }
      mir.basic_blocks_mut()[index] = data.clone();
    }
  }
}
