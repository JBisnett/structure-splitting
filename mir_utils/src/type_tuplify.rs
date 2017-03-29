use rustc::ty::TyCtxt;
use rustc::mir;
use rustc::mir::visit;
use rustc::mir::visit::Visitor;

use split_struct::LocalMap;

use std::collections::hash_map::HashMap;
