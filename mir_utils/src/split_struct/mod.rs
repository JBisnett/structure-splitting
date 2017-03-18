extern crate rustc;
extern crate syntax;

use rustc::ty::TyCtxt;
use rustc::ty;

use std::collections::HashMap;
use std::vec::Vec;

use syntax::ext::base::{ExtCtxt, Annotatable};
use syntax::ast;
use syntax::ext::build::AstBuilder;
use syntax::codemap::DUMMY_SP;

#[derive(Clone, Debug)]
pub struct SplitStruct {
  pub name: String,
  pub child_names: Vec<String>,
  // field index -> (child name, field index for child)
  pub field_map: HashMap<usize, (String, usize)>,
}

impl SplitStruct {
  pub fn process_declarations(
    ex: &mut ExtCtxt,
    name: String,
         declarations: HashMap<usize,
                               Vec<(usize, &syntax::ast::StructField)>>)
         -> (Self, Vec<Annotatable>) {
    let mut created_structs = vec![];
    let mut child_names = vec![];
    let mut field_map = HashMap::new();
    for (affinity, fields_enumerated) in declarations {
      let child_struct_name = String::new() + &*name + &*affinity.to_string();
      child_names.push(child_struct_name.clone());
      for &(field_number, _) in fields_enumerated.iter() {
        let field_index = field_map.values()
          .filter(|&&(ref name, _)| *name == child_struct_name)
          .count();
        field_map.insert(field_number,
                         (child_struct_name.clone(), field_index));
      }
      let fields = fields_enumerated.iter().map(|&(_, v)| v.clone()).collect();
      let x = ex.item_struct(DUMMY_SP,
                             ast::Ident::from_str(&*child_struct_name),
                             ast::VariantData::Struct(fields,
                                                      ast::DUMMY_NODE_ID));
      created_structs.push(Annotatable::Item(x));
    }
    created_structs.push(Annotatable::Item(quote_item!{ex,
      struct TestStandin {
        pub index: i64,
      }
    }
      .unwrap()));
    created_structs.push(Annotatable::Item(quote_item!{ex,
                  impl Drop for TestStandin {
                    fn drop(&mut self) {}
                  }
    }
      .unwrap()));
    (Self {
      name: name,
      child_names: child_names,
      field_map: field_map,
    },
     created_structs)
  }
}

pub fn make_split_ty_map<'a, 'tcx>(tcx: TyCtxt<'a, 'tcx, 'tcx>,
                                  string_map: &HashMap<String, SplitStruct>)
    -> (HashMap<ast::NodeId, Vec<ast::NodeId>>,
        HashMap<ty::Ty<'tcx>, SplitStruct>)
{
  let mut split_map = HashMap::new();
  let mut ty2structsplit = HashMap::new();
  for (split_struct_name, child_split) in string_map.iter() {
    for nodeid in tcx.hir
      .nodes_matching_suffix(&[split_struct_name.to_string()]) {
      let ty = tcx.item_type(tcx.hir.local_def_id(nodeid));
      ty2structsplit.insert(ty, child_split.clone());
      for child_name in child_split.child_names.iter() {
        for child_id in tcx.hir.nodes_matching_suffix(&[child_name.clone()]) {
          split_map.entry(nodeid).or_insert(vec![]).push(child_id);
        }
      }
    }
  }
  (split_map, ty2structsplit)
}
