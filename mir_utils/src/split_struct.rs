extern crate rustc;
extern crate syntax;

use walkmut::{TypeModifier, SplitTypeModifier};

use rustc::ty::TyCtxt;
use rustc::ty;
use rustc::mir;

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
  pub tupName: String,
  pub refTupName: String,
  pub vecTupName: String,
}

impl SplitStruct {
  pub fn process_declarations(ex: &mut ExtCtxt,
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
        field_map.insert(field_number, (child_struct_name.clone(), field_index));
      }
      let fields = fields_enumerated.iter().map(|&(_, v)| v.clone()).collect();
      let x = ex.item_struct(DUMMY_SP,
                             ast::Ident::from_str(&*child_struct_name),
                             ast::VariantData::Struct(fields, ast::DUMMY_NODE_ID));
      created_structs.push(x);
    }
    let child1 = child_names[0].clone();
    let child2 = child_names[1].clone();
    let tup = name.clone() + "tup";
    let rtup = name.clone() + "Rtup";
    let vtup = name.clone() + "Vtup";
    //created_structs.push(quote_item!{ex, struct $tup($child1, $child2);}.unwrap());
    (Self {
       name: name,
       child_names: child_names,
       field_map: field_map,
       tupName: tup,
       refTupName: rtup,
       vecTupName: vtup,
     },
     created_structs.into_iter().map(Annotatable::Item).collect())
  }
}

pub type NodeMap = HashMap<ast::NodeId, Vec<ast::NodeId>>;
pub type SplitMap<'tcx> = HashMap<ty::Ty<'tcx>, SplitStruct>;
pub type LocalMap<'tcx> = HashMap<mir::Local, HashMap<ty::Ty<'tcx>, mir::Local>>;
pub type TypeMap<'tcx> = HashMap<ty::Ty<'tcx>, ty::Ty<'tcx>>;

pub fn get_ty_map<'a, 'tcx>(tcx: TyCtxt<'a, 'tcx, 'tcx>,
                            string_map: &HashMap<String, SplitStruct>)
                            -> TypeMap<'tcx> {
  let mut ty_map = HashMap::new();
  for (split_struct_name, child_split) in string_map.iter() {
    let ids = ["", "TUP", "RTUP", "ATUP", "ARTUP", "VTUP", "VRTUP"]
      .into_iter()
      .map(|additional| {
        let mut id = None;
        let mut ty = None;
        for nodeid in tcx.hir
          .nodes_matching_suffix(&[split_struct_name.to_string() + additional]) {
          id = Some(nodeid);
          ty = Some(tcx.item_type(tcx.hir.local_def_id(nodeid)));
        }
        (id.unwrap(), ty.unwrap())
      });
    let (nodeid, nodety) = ids.next().unwrap();
    let (tupid, tupty) = ids.next().unwrap();
    let (rtupid, rtupty) = ids.next().unwrap();
    let (atupid, atupty) = ids.next().unwrap();
    let (artupid, artupty) = ids.next().unwrap();
    let (vtupid, vtupty) = ids.next().unwrap();
    let (vrtupid, vrtupty) = ids.next().unwrap();
    ty_map.insert(nodety, tupty);
  }
  ty_map
}

pub fn make_split_ty_map<'a, 'tcx>(tcx: TyCtxt<'a, 'tcx, 'tcx>,
                                   string_map: &HashMap<String, SplitStruct>)
                                   -> (NodeMap, SplitMap<'tcx>) {
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

pub fn make_decl_map<'a, 'tcx, 'b>(tcx: TyCtxt<'a, 'tcx, 'tcx>,
                                   mir: &mut mir::Mir<'tcx>,
                                   split_map: &'b HashMap<ast::NodeId, Vec<ast::NodeId>>)
                                   -> LocalMap<'tcx> {
  let mut decl_map = HashMap::new();
  for (local, decl) in mir.local_decls.clone().into_iter_enumerated() {
    // println!{"{:?}: {:?}", local, decl};
    for nty in decl.ty.walk() {
      if let ty::TyAdt(adt, _) = nty.sty {
        if let Some(node_id) = tcx.hir.as_local_node_id(adt.did) {
          if let Some(child_ids) = split_map.get(&node_id) {
            for child_id in child_ids {
              let ty = tcx.item_type(tcx.hir.local_def_id(*child_id));
              if let ty::TyAdt(new_adt, _) = ty.sty {
                let mut type_modifier = SplitTypeModifier::new(adt, new_adt);
                if let Ok(new_ty) = type_modifier.modify(tcx, decl.ty) {
                  // println!{"{:?} -> {:?}", decl.ty, new_ty};
                  let child_decl = mir::LocalDecl::new_temp(new_ty);
                  let child_local = mir.local_decls.push(child_decl);
                  decl_map.entry(local)
                    .or_insert(HashMap::new())
                    .insert(ty, child_local);
                }
              }
            }
          } else {
            // println!{"Non-split ADT is detected"};
          }
        }
      }
    }
  }
  // println!{};
  decl_map
}
