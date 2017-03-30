use rustc::ty::TyCtxt;
use rustc::mir;
use rustc::mir::visit;
use rustc::mir::visit::Visitor;

use rustc_data_structures::indexed_vec::Idx;

use split_struct::{SplitMap, LocalMap};

#[derive(new)]
pub struct StructFieldReplacer<'a, 'tcx: 'a + 'mr, 'mr> {
  tcx: TyCtxt<'a, 'tcx, 'tcx>,
  mir: &'mr mir::Mir<'tcx>,
  ty2structsplit: &'a SplitMap<'tcx>,
  decl_map: &'a LocalMap<'tcx>,
}

struct LocalFinder {
  local: Option<mir::Local>,
}

// THIS DEPENDS ON RECURSION DOWN THE BASE OF PROJECTION
impl<'tcx> visit::Visitor<'tcx> for LocalFinder {
  fn visit_lvalue(&mut self,
                  lvalue: &mir::Lvalue<'tcx>,
                  context: visit::LvalueContext<'tcx>,
                  location: mir::Location) {
    if None == self.local {
      if let &mir::Lvalue::Local(local) = lvalue {
        self.local = Some(local);
      } else {
        self.super_lvalue(lvalue, context, location);
      }
    }
  }
}

#[derive(new)]
struct LocalReplacer {
  old: mir::Local,
  new: mir::Local,
}

impl<'tcx> visit::MutVisitor<'tcx> for LocalReplacer {
  fn visit_lvalue(&mut self,
                  lvalue: &mut mir::Lvalue<'tcx>,
                  context: visit::LvalueContext<'tcx>,
                  location: mir::Location) {
    if let &mut mir::Lvalue::Local(local) = lvalue {
      if local == self.old {
        *lvalue = mir::Lvalue::Local(self.new);
      }
    } else {
      self.super_lvalue(lvalue, context, location);
    }
  }
}

impl<'a, 'tcx, 'mr> visit::MutVisitor<'tcx>
  for StructFieldReplacer<'a, 'tcx, 'mr> {
  fn visit_projection(&mut self,
                      projection: &mut mir::LvalueProjection<'tcx>,
                      context: mir::visit::LvalueContext<'tcx>,
                      location: mir::Location) {
    // println!{"{:?}, {:?} {:?}", projection, context, location}
    let base_ty = projection.base.ty(&self.mir, self.tcx).to_ty(self.tcx);
    if let Some(split_struct) = self.ty2structsplit.get(base_ty) {
      if let mir::ProjectionElem::Field(field, field_ty) = projection.elem {
        let (ref target_string, index) = split_struct.field_map[&field.index()];
        for target_node in self.tcx
          .hir
          .nodes_matching_suffix(&[target_string.clone()]) {
          let child_ty = self.tcx
            .item_type(self.tcx.hir.local_def_id(target_node));
          let mut local_visitor = LocalFinder { local: None };
          local_visitor.visit_lvalue(&projection.base, context, location);
          let base_local = local_visitor.local.unwrap();
          if let Some(target_local) = self.decl_map[&base_local].get(child_ty) {
            LocalReplacer::new(base_local, *target_local)
              .visit_lvalue(&mut projection.base, context, location);
            projection.elem = mir::ProjectionElem::Field(mir::Field::new(index),
                                                         field_ty)
          }
        }
      }
    }
  }
}
