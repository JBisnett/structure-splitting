use syntax;
use rustc::ty::TyCtxt;
use rustc::ty;
use rustc::mir;
use rustc::mir::visit;
use rustc_data_structures::indexed_vec::Idx;

use split_struct::LocalMap;
use split_utils;

trait Splittable {}
