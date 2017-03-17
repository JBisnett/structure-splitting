use rustc::ty;
use rustc::ty::TyCtxt;
use rustc::mir;
use rustc::mir::visit;
use rustc::mir::visit::Visitor;

use rustc_data_structures::indexed_vec::Idx;

use split_struct::SplitStruct;

use std::collections::hash_map::HashMap;

pub mod deaggregator;

pub struct ExtenderMirPass<'tcx> {
  extender: Box<FnMut(mir::BasicBlock, mir::Statement<'tcx>, mir::Location)
                      -> Vec<mir::Statement>>,
}

impl<'tcx> ExtenderMirPass<'tcx> {
  pub fn new(extender: Box<FnMut(mir::BasicBlock,
                                 mir::Statement<'tcx>,
                                 mir::Location)
                                 -> Vec<mir::Statement>>) {
    ExtenderMirPass { extender: extender };
  }
}

impl<'tcx> visit::MutVisitor<'tcx> for ExtenderMirPass<'tcx> {
  fn visit_basic_block_data(&mut self,
                            block: mir::BasicBlock,
                            data: &mut mir::BasicBlockData<'tcx>) {
    let mut new_statements = vec![];
    let mut index = 0;
    for statement in data.statements.clone().iter() {
      let location = mir::Location {
        block: block,
        statement_index: index,
      };
      let extended = (self.extender)(block, statement.clone(), location);
      new_statements.extend_from_slice(&*extended);
      index += 1;
    }
    data.statements = new_statements;
  }
}

#[derive(new)]
pub struct StructFieldReplacer<'a, 'tcx: 'a, 'v> {
  tcx: TyCtxt<'a, 'tcx, 'tcx>,
  split_info: &'v SplitStruct,
  decl_map: &'a HashMap<&'a ty::AdtDef, mir::Local>,
}


impl<'a, 'tcx, 'v> visit::MutVisitor<'tcx>
  for StructFieldReplacer<'a, 'tcx, 'v> {
  fn visit_projection(&mut self,
                      projection: &mut mir::LvalueProjection<'tcx>,
                      context: mir::visit::LvalueContext<'tcx>,
                      location: mir::Location) {
    println!{"{:?} {:?} {:?}", projection, context, location};
    if let mir::ProjectionElem::Field(field, ty) = projection.elem {
      let (ref target_string, index) = self.split_info
        .field_map[&field.index()];
      println!{"{:?}[{:?}]",
               target_string, index};
      for target_node in self.tcx
        .hir
        .nodes_matching_suffix(&[target_string.clone()]) {
        if let ty::TypeVariants::TyAdt(adt, _) = self.tcx
          .item_type(self.tcx.hir.local_def_id(target_node))
          .sty {
          let target_local = self.decl_map[adt];
          projection.base = mir::Lvalue::Local(target_local);
          projection.elem = mir::ProjectionElem::Field(mir::Field::new(index),
                                                       ty);
        }
      }
    }
    println!{};
    self.super_projection(projection, context, location);
  }
  fn visit_lvalue(&mut self,
                  lvalue: &mut mir::Lvalue<'tcx>,
                  context: visit::LvalueContext<'tcx>,
                  location: mir::Location) {
    self.super_lvalue(lvalue, context, location);
  }
}

#[derive(new)]
struct LvalueReplacer {
  old: mir::Local,
  new: mir::Local,
}

impl<'tcx> visit::MutVisitor<'tcx> for LvalueReplacer {
  fn visit_lvalue(&mut self,
                  lvalue: &mut mir::Lvalue<'tcx>,
                  _: visit::LvalueContext<'tcx>,
                  _: mir::Location) {
    if let &mut mir::Lvalue::Local(local) = lvalue {
      if local == self.old {
        *lvalue = mir::Lvalue::Local(self.new);
      }
    }
  }
}

struct LvalueFinder<'a, 'tcx: 'a> {
  decl_maps: &'a HashMap<mir::Local, HashMap<&'a ty::AdtDef, mir::Local>>,
  value: Option<mir::Lvalue<'tcx>>,
}

impl<'a, 'tcx> LvalueFinder<'a, 'tcx> {
  fn new(decl_maps: &'a HashMap<mir::Local,
                                HashMap<&'a ty::AdtDef, mir::Local>>)
         -> Self {
    LvalueFinder {
      decl_maps: decl_maps,
      value: None,
    }
  }
}

impl<'a, 'tcx> visit::Visitor<'tcx> for LvalueFinder<'a, 'tcx> {
  fn visit_assign(&mut self,
                  _: mir::BasicBlock,
                  lvalue: &mir::Lvalue<'tcx>,
                  _: &mir::Rvalue<'tcx>,
                  _: mir::Location) {
    if let &mir::Lvalue::Local(local) = lvalue {
      if self.decl_maps.get(&local) != None {
        self.value = Some(mir::Lvalue::Local(local))
      }
    }
  }
}

#[derive(new)]
pub struct StructLvalueSplitter<'a> {
  decl_maps: &'a HashMap<mir::Local, HashMap<&'a ty::AdtDef, mir::Local>>,
}

impl<'a, 'tcx, 'v> visit::MutVisitor<'tcx> for StructLvalueSplitter<'a> {
  fn visit_basic_block_data(&mut self,
                            block: mir::BasicBlock,
                            data: &mut mir::BasicBlockData<'tcx>) {
    let mut new_statements = vec![];
    let mut index = 0;
    for statement in data.statements.clone().iter() {
      let location = mir::Location {
        block: block,
        statement_index: index,
      };
      index += 1;
      let mut visitors = vec![];
      let mut finder = LvalueFinder::new(self.decl_maps);
      finder.visit_statement(block, statement, location);
      if let Some(mir::Lvalue::Local(local)) = finder.value {
        for alt_local in self.decl_maps[&local].values() {
          visitors.push(LvalueReplacer::new(local, *alt_local));
        }
      }
      if visitors.len() == 0 {
        new_statements.push(statement.clone());
      }
      for mut visitor in visitors {
        let mut new_statement = statement.clone();
        visitor.visit_statement(block, &mut new_statement, location);
        new_statements.push(new_statement);
      }
    }
    data.statements = new_statements;
    self.super_basic_block_data(block, data);
  }
}
