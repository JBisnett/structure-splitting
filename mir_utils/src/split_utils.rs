use rustc::ty::TyCtxt;
use rustc::ty;
use rustc::hir;
use rustc::mir;
use rustc_data_structures::indexed_vec::Idx;
use rustc_data_structures::indexed_vec::IndexVec;

use std::collections::HashMap;
fn get_type_list<'a, 'tcx>(tcx: TyCtxt<'a, 'tcx, 'tcx>,
                           local_decls: IndexVec<mir::Local,
                                                 mir::LocalDecl<'tcx>>,
                           local_vec: Vec<mir::Local>)
                           -> Vec<ty::Ty<'tcx>> {
  let mut ty_vec = vec![];
  for ref new_local in local_vec.clone() {
    ty_vec.push(local_decls[*new_local].ty);
  }
  let new_type_list = tcx.intern_type_list(&*ty_vec);
  new_type_list.to_vec()
}

pub fn get_assignments_for_local<'tcx, 'v>
  (local: mir::Local,
   new_type_list: &'v [ty::Ty<'tcx>],
   local_vec: &[mir::Local])
   -> Vec<(mir::Lvalue<'tcx>, mir::Lvalue<'tcx>)> {
  let assigns = local_vec.iter()
    .enumerate()
    .map(|(i, l)| {
      let tuple_projection = mir::Lvalue::Local(local)
        .field(mir::Field::new(i), new_type_list[i]);
      let lhs = mir::Lvalue::Local(*l);
      let rhs = tuple_projection;
      (lhs, rhs)
    })
    .collect();
  assigns
}

fn get_local_map<'v>(local_map: &'v HashMap<ty::Ty, mir::Local>)
                     -> Vec<mir::Local> {
  let mut adt_local_vec: Vec<_> = local_map.iter()
    .collect::<Vec<_>>();
  adt_local_vec.sort_by_key(|&(t, _)| {
    if let ty::TypeVariants::TyAdt(adt, _) = t.sty {
      adt.did
    } else {
      panic!{"Local Type Varient isn't an adt"};
      hir::def_id::DefId::local(hir::def_id::CRATE_DEF_INDEX)
    }
  });
  adt_local_vec.iter()
    .map(|&(_, t)| *t)
    .collect()
}

pub fn get_split_structures<'a, 'tcx, 'v>
  (tcx: TyCtxt<'a, 'tcx, 'tcx>,
   local: mir::Local,
   local_decls: IndexVec<mir::Local, mir::LocalDecl<'tcx>>,
   local_map: &'v HashMap<ty::Ty, mir::Local>)
   -> (Vec<ty::Ty<'tcx>>, Vec<(mir::Lvalue<'tcx>, mir::Lvalue<'tcx>)>) {
  let local_vec = get_local_map(local_map);
  let tys = get_type_list(tcx, local_decls, local_vec.clone());
  let vals = get_assignments_for_local(local, &*tys, &*local_vec);
  (tys, vals)
}
