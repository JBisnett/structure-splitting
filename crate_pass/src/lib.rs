#![feature(plugin_registrar, rustc_private)]
#![feature(box_syntax)]
#![feature(quote, plugin_registrar, rustc_private)]

extern crate rustc;
extern crate rustc_mir;
extern crate rustc_data_structures;
extern crate rustc_plugin;
extern crate rustc_const_math;
extern crate syntax;

extern crate mir_utils;

// convenience crates
#[macro_use]
extern crate lazy_static;

use rustc::mir::transform::{self, MirPass, MirSource};
use rustc::ty::TyCtxt;
use rustc::mir::Mir;
use rustc::mir::visit::MutVisitor;

use rustc_plugin::Registry;

use syntax::ast;
use syntax::ext::base::{ExtCtxt, SyntaxExtension, Annotatable};
use syntax::codemap::Span;
use syntax::symbol::Symbol;

use std::collections::HashMap;

use std::sync::Mutex;

use mir_utils::split_struct::{make_split_ty_map, make_decl_map, SplitStruct};
use mir_utils::lvalue_splitter::StructLvalueSplitter;
use mir_utils::struct_base_replacer::StructFieldReplacer;
use mir_utils::deaggregator::Deaggregator;
use mir_utils::split_arg::ArgumentSplitter;

struct StructureSplitting;
impl transform::Pass for StructureSplitting {}

// TODO: verify the find with attributes, not just name
impl<'tcx> MirPass<'tcx> for StructureSplitting {
  fn run_pass<'a>(&mut self,
                  tcx: TyCtxt<'a, 'tcx, 'tcx>,
                  source: MirSource,
                  mir: &mut Mir<'tcx>) {
    Deaggregator.run_pass(tcx, source, mir);

    let string_map = SPLIT_STRUCTS.lock().unwrap();

    let (split_map, ty2structsplit) = make_split_ty_map(tcx, &*string_map);
    let decl_map = make_decl_map(tcx, mir, &split_map);
    let mir_copy = mir.clone();
    println!{"{:?}", decl_map};
    {
      let mut arg_tupler = ArgumentSplitter::new(tcx, &decl_map);
      arg_tupler.visit_mir(mir);
    }
    {
      let mut visitor =
        StructFieldReplacer::new(tcx, &mir_copy, &ty2structsplit, &decl_map);
      visitor.visit_mir(mir);
    }
    {
      let mut assignment_splitter =
        StructLvalueSplitter::new(tcx, &mir_copy, &decl_map);
      assignment_splitter.visit_mir(mir);
    }
  }
}

fn expand(ex: &mut ExtCtxt,
          _: Span,
          meta: &ast::MetaItem,
          item: Annotatable)
          -> Vec<Annotatable> {
  if let ast::Item { ident, node: ast::ItemKind::Struct(ref data, _), .. } =
         *item.clone()
    .expect_item() {
    let mut declarations = HashMap::new();
    let field_set = data.fields()
      .iter()
      .enumerate()
      .map(|x| (x.1.ident.unwrap().name.as_str(), x))
      .collect::<HashMap<_, _>>();
    if let ast::MetaItemKind::List(ref list) = meta.node {
      for field in list {
        if let
            ast::NestedMetaItemKind::MetaItem(
              ast::MetaItem {
                node: ast::MetaItemKind::NameValue(ref affinity_context),
                name: ref field_name, .. }) = field.node {
              if !field_set.contains_key(&field_name.as_str()) {
                ex.span_err(field.span,
                            "Setting affinity group of struct \
                             field that does not exist")
              }
              if let ast::LitKind::Int(affinity_group,
                                  ast::LitIntType::Unsuffixed) =
                affinity_context.node {
                  declarations.entry(affinity_group as usize)
                    .or_insert(vec![])
                    .push(field_set[&field_name.as_str()]
                          .clone());
                } else {
                  ex.span_err(field.span,
                              "Affinity group must be an \
                               unsuffixed integer");
                }
            }
      }
    }
    let struct_name = &*ident.name.as_str();
    let mut shared_struct_table = SPLIT_STRUCTS.lock()
      .unwrap();
    let (split_string_obj, mut created_structs) =
      SplitStruct::process_declarations(ex,
                                        struct_name.to_string(),
                                        declarations);
    shared_struct_table.entry(struct_name.to_string())
      .or_insert(split_string_obj);
    // Loop throught fields of shared_structs
    created_structs.push(item);
    created_structs
  } else {
    vec![item]
  }
}

lazy_static! {
  static ref SPLIT_STRUCTS : Mutex<HashMap<String, SplitStruct>> =
    Mutex::new(HashMap::new());
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
  reg.register_mir_pass(box StructureSplitting);
  let b_expand = SyntaxExtension::MultiModifier(box expand);
  reg.register_syntax_extension(Symbol::intern("affinity_groups"), b_expand);
}
