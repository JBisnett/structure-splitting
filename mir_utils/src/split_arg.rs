use syntax;
use rustc::ty::TyCtxt;
use rustc::ty;
use rustc::hir;
use rustc::mir;
use rustc::mir::visit;
use rustc_data_structures::indexed_vec::Idx;
use rustc_data_structures::indexed_vec::IndexVec;

use split_struct::LocalMap;

#[derive(new)]
pub struct ArgumentSplitter<'a, 'tcx: 'a> {
  tcx: TyCtxt<'a, 'tcx, 'tcx>,
  decl_maps: &'a LocalMap<'tcx>,
}

impl<'a, 'tcx> ArgumentSplitter<'a, 'tcx> {}


impl<'a, 'tcx> visit::MutVisitor<'tcx> for ArgumentSplitter<'a, 'tcx> {
  fn visit_mir(&mut self, mir: &mut mir::Mir<'tcx>) {
    let source_info = mir::SourceInfo {
      span: syntax::codemap::DUMMY_SP,
      scope: mir::ARGUMENT_VISIBILITY_SCOPE,
    };
    for local in mir.args_iter() {
      if let Some(local_map) = self.decl_maps.get(&local) {
        let mut adt_local_vec: Vec<_> = local_map.iter()
          .collect::<Vec<_>>();
        adt_local_vec.sort_by_key(|&(t, _)| {
          if let ty::TypeVariants::TyAdt(adt, _) = t.sty {
            adt.did
          } else {
            println!{"THIS SHOULD NOT HAPPEN"};
            hir::def_id::DefId::local(hir::def_id::CRATE_DEF_INDEX)
          }
        });
        let local_vec: Vec<mir::Local> = adt_local_vec.iter()
          .map(|&(_, t)| *t)
          .collect();
        let mut ty_vec = vec![];
        for ref new_local in local_vec.clone() {
          ty_vec.push(mir.local_decls[*new_local].ty);
        }
        let new_type_list = self.tcx.intern_type_list(&*ty_vec);
        mir.local_decls[local].ty = self.tcx
          .mk_ty(ty::TyTuple(new_type_list, false));
        let mut assigns: Vec<mir::Statement> = local_vec.iter()
          .enumerate()
          .map(|(i, l)| {
            let tuple_projection = mir::Lvalue::Local(local)
              .field(mir::Field::new(i), new_type_list[i]);
            let rhs = mir::Rvalue::Use(mir::Operand::Consume(tuple_projection));
            let lhs = mir::Lvalue::Local(*l);
            let assign_kind = mir::StatementKind::Assign(lhs, rhs);
            let statement = mir::Statement {
              source_info: source_info,
              kind: assign_kind,
            };
            statement
          })
          .collect();
        // Add assigns to the front of the first basic block
        {
          let ref first_block = mir.basic_blocks()[mir::START_BLOCK].statements;
          assigns.extend_from_slice(&first_block);
        }
        mir.basic_blocks_mut()[mir::START_BLOCK].statements = assigns;
      }
    }
    self.super_mir(mir);
  }
}
