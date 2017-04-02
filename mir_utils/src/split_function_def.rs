use syntax;
use rustc::ty::TyCtxt;
use rustc::mir;
use rustc::mir::visit;
use rustc_data_structures::indexed_vec::Idx;

use split_struct::LocalMap;
use split_utils;

#[derive(new)]
pub struct SignatureSplitter<'a, 'tcx: 'a> {
  tcx: TyCtxt<'a, 'tcx, 'tcx>,
  decl_maps: &'a LocalMap<'tcx>,
}

impl<'a, 'tcx> visit::MutVisitor<'tcx> for SignatureSplitter<'a, 'tcx> {
  fn visit_mir(&mut self, mir: &mut mir::Mir<'tcx>) {
    let source_info = mir::SourceInfo {
      span: syntax::codemap::DUMMY_SP,
      scope: mir::ARGUMENT_VISIBILITY_SCOPE,
    };
    for local in mir.args_iter() {
      if let Some(local_map) = self.decl_maps.get(&local) {
        let (tys, vals) = split_utils::get_split_structures(self.tcx,
                                                            local,
                                                            mir.local_decls
                                                              .clone(),
                                                            local_map);
        mir.local_decls[local].ty = self.tcx.intern_tup(&*tys, false);
        // Add assigns to the front of the first basic block
        let mut assigns = vals.into_iter()
          .map(|(local_temp, tuple_projection)| {
            let rhs = mir::Rvalue::Use(mir::Operand::Consume(tuple_projection));
            let assign_kind = mir::StatementKind::Assign(local_temp, rhs);
            mir::Statement {
              source_info: source_info,
              kind: assign_kind,
            }
          })
          .collect::<Vec<mir::Statement>>();
        {
          let ref first_block = mir.basic_blocks()[mir::START_BLOCK].statements;
          assigns.extend_from_slice(&first_block);
        }
        mir.basic_blocks_mut()[mir::START_BLOCK].statements = assigns;
      }
    }
    let return_local = mir::Local::new(0);
    if let Some(local_map) = self.decl_maps.get(&return_local) {
      let (tys, vals) = split_utils::get_split_structures(self.tcx,
                                                          return_local,
                                                          mir.local_decls
                                                            .clone(),
                                                          local_map);
      let tuple_ty = self.tcx.intern_tup(&*tys, false);
      mir.local_decls[return_local].ty = tuple_ty;
      mir.return_ty = tuple_ty;
      let assigns = vals.into_iter()
        .map(|(local_temp, tuple_projection)| {
          let rhs = mir::Rvalue::Use(mir::Operand::Consume(local_temp));
          let assign_kind = mir::StatementKind::Assign(tuple_projection, rhs);
          mir::Statement {
            source_info: source_info,
            kind: assign_kind,
          }
        })
        .collect::<Vec<mir::Statement>>();
      let return_blocks = mir.basic_blocks_mut()
        .iter_mut()
        .filter(|block| if let mir::TerminatorKind::Return = block.terminator()
          .kind {
          true
        } else {
          false
        });
      for mut block in return_blocks {
        block.statements.extend(assigns.clone());
      }
    }
    self.super_mir(mir);
  }
}
