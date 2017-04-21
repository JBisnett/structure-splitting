use rustc::ty::{self, Ty, TyCtxt};
use rustc::ty::subst;
use rustc::ty::fold::TypeFolder;
use rustc::ty::TypeFoldable;
use rustc::hir;
// possibly move over to using TypeFolders
pub trait TypeModifier<'a, 'tcx> {
  fn modify_array(&mut self,
                  tcx: TyCtxt<'a, 'tcx, 'tcx>,
                  ty: Ty<'tcx>,
                  size: usize)
                  -> Result<Ty<'tcx>, ()> {
    let new_ty = self.modify(tcx, ty)?;
    Ok(tcx.mk_array(new_ty, size))
  }
  fn modify_slice(&mut self, tcx: TyCtxt<'a, 'tcx, 'tcx>, ty: Ty<'tcx>) -> Result<Ty<'tcx>, ()> {
    let new_ty = self.modify(tcx, ty)?;
    Ok(tcx.mk_slice(new_ty))
  }
  fn modify_raw_ptr(&mut self,
                    _: TyCtxt<'a, 'tcx, 'tcx>,
                    _: &ty::TypeAndMut<'tcx>)
                    -> Result<Ty<'tcx>, ()> {
    Err(())
  }
  fn modify_ref(&mut self,
                tcx: TyCtxt<'a, 'tcx, 'tcx>,
                region: &'tcx ty::Region,
                ty: &ty::TypeAndMut<'tcx>)
                -> Result<Ty<'tcx>, ()> {
    let new_ty = self.modify(tcx, ty.ty)?;
    let mut new_ty_mut = ty.clone();
    new_ty_mut.ty = new_ty;
    Ok(tcx.mk_ref(region, new_ty_mut))
  }
  fn modify_projection(&mut self,
                       _: TyCtxt<'a, 'tcx, 'tcx>,
                       _: &ty::ProjectionTy<'tcx>)
                       -> Result<Ty<'tcx>, ()> {
    Err(())
  }
  fn modify_dynamic(&mut self,
                    _: TyCtxt<'a, 'tcx, 'tcx>,
                    _: &ty::Binder<&'tcx ty::Slice<ty::ExistentialPredicate<'tcx>>>,
                    _: &'tcx ty::Region)
                    -> Result<Ty<'tcx>, ()> {
    Err(())
  }
  fn modify_adt(&mut self,
                _: TyCtxt<'a, 'tcx, 'tcx>,
                _: &'tcx ty::AdtDef,
                _: &'tcx subst::Substs<'tcx>)
                -> Result<Ty<'tcx>, ()> {
    Err(())
  }

  fn modify_anon(&mut self,
                 _: TyCtxt<'a, 'tcx, 'tcx>,
                 _: hir::def_id::DefId,
                 _: &'tcx subst::Substs<'tcx>)
                 -> Result<Ty<'tcx>, ()> {
    Err(())
  }

  fn modify_closure(&mut self,
                    _: TyCtxt<'a, 'tcx, 'tcx>,
                    _: hir::def_id::DefId,
                    _: ty::ClosureSubsts<'tcx>)
                    -> Result<Ty<'tcx>, ()> {
    Err(())
  }

  fn modify_tuple(&mut self,
                  tcx: TyCtxt<'a, 'tcx, 'tcx>,
                  tys: &'tcx ty::Slice<Ty<'tcx>>,
                  default: bool)
                  -> Result<Ty<'tcx>, ()> {
    let mut new_tys = vec![];
    let mut is_changed = false;
    for ty in tys.iter() {
      if let Ok(new_ty) = self.modify(tcx, ty) {
        new_tys.push(new_ty);
        is_changed = true;
      } else {
        new_tys.push(ty);
      }
    }
    if is_changed {
      let type_list = tcx.intern_type_list(&*new_tys);
      Ok(tcx.mk_ty(ty::TyTuple(type_list, default)))
    } else {
      Err(())
    }
  }

  fn modify_fn_def(&mut self,
                   _: TyCtxt<'a, 'tcx, 'tcx>,
                   _: hir::def_id::DefId,
                   _: &'tcx subst::Substs<'tcx>,
                   _: ty::PolyFnSig<'tcx>)
                   -> Result<Ty<'tcx>, ()> {
    Err(())
  }

  fn modify_fn_ptr(&mut self,
                   _: TyCtxt<'a, 'tcx, 'tcx>,
                   _: ty::PolyFnSig<'tcx>)
                   -> Result<Ty<'tcx>, ()> {
    Err(())
  }

  fn modify(&mut self, tcx: TyCtxt<'a, 'tcx, 'tcx>, parent_ty: Ty<'tcx>) -> Result<Ty<'tcx>, ()> {
    match parent_ty.sty {
      ty::TyBool | ty::TyChar | ty::TyInt(_) | ty::TyUint(_) | ty::TyFloat(_) | ty::TyStr |
      ty::TyInfer(_) | ty::TyParam(_) | ty::TyNever | ty::TyError => Err(()),
      ty::TyArray(ty, size) => self.modify_array(tcx, ty, size),
      ty::TySlice(ty) => self.modify_slice(tcx, ty),
      ty::TyRawPtr(ref mt) => self.modify_raw_ptr(tcx, mt),
      ty::TyRef(ref region, ref mt) => self.modify_ref(tcx, region, mt),
      ty::TyProjection(ref data) => self.modify_projection(tcx, data),
      ty::TyDynamic(ref obj, ref region) => self.modify_dynamic(tcx, obj, region),
      ty::TyAdt(adt, substs) => self.modify_adt(tcx, adt, substs),
      ty::TyAnon(did, substs) => self.modify_anon(tcx, did, substs),
      ty::TyClosure(did, substs) => self.modify_closure(tcx, did, substs),
      ty::TyTuple(ts, default) => self.modify_tuple(tcx, ts, default),
      ty::TyFnDef(did, substs, ft) => self.modify_fn_def(tcx, did, substs, ft),
      ty::TyFnPtr(ft) => self.modify_fn_ptr(tcx, ft),
    }
  }
}

#[derive(new)]
pub struct StructTypeModifier<'tcx> {
  old: &'tcx ty::AdtDef,
  new: &'tcx ty::AdtDef,
}

impl<'a, 'tcx> TypeModifier<'a, 'tcx> for StructTypeModifier<'tcx> {
  fn modify_adt(&mut self,
                tcx: TyCtxt<'a, 'tcx, 'tcx>,
                adt: &'tcx ty::AdtDef,
                substs: &'tcx subst::Substs<'tcx>)
                -> Result<Ty<'tcx>, ()> {
    if adt == self.old {
      Ok(tcx.mk_adt(self.new, substs))
    } else {
      Err(())
    }
  }
}

// Use this somewhere
pub struct StructWalker<'a,
                        'gcx: 'a + 'tcx,
                        'tcx: 'a,
                        F: FnMut(&'tcx ty::AdtDef) -> &'tcx ty::AdtDef>
{
  tcx: TyCtxt<'a, 'gcx, 'tcx>,
  st_func: F,
}

impl<'a, 'gcx, 'tcx, F> TypeFolder<'gcx, 'tcx> for StructWalker<'a, 'gcx, 'tcx, F>
  where F: FnMut(&'tcx ty::AdtDef) -> &'tcx ty::AdtDef
{
  fn tcx<'b>(&'b self) -> TyCtxt<'b, 'gcx, 'tcx> {
    self.tcx
  }

  fn fold_ty(&mut self, t: Ty<'tcx>) -> Ty<'tcx> {
    if let ty::TypeVariants::TyAdt(adt, substs) = t.sty {
      let new_sty = ty::TypeVariants::TyAdt((self.st_func)(adt), substs.fold_with(self));
      self.tcx().mk_ty(new_sty)
    } else {
      t.super_fold_with(self)
    }
  }
}
