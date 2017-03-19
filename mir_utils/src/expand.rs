use rustc::ty;
use rustc::ty::TyCtxt;
use rustc::mir;
use rustc::mir::visit;
use rustc::mir::visit::Visitor;

use rustc_data_structures::indexed_vec::Idx;

use split_struct::SplitStruct;

use std::collections::hash_map::HashMap;

#[derive(new)]
pub struct StructFieldReplacer<'a, 'tcx: 'a> {
  tcx: TyCtxt<'a, 'tcx, 'tcx>,
  mir: &'tcx mir::Mir<'tcx>,
  split_map: HashMap<ty::Ty<'tcx>, SplitStruct>,
  decl_map: HashMap<mir::Local, HashMap<&'a ty::AdtDef, mir::Local>>,
}


impl<'a, 'tcx, 'v> visit::MutVisitor<'tcx> for StructFieldReplacer<'a, 'tcx> {
  fn visit_projection(&mut self,
                      projection: &mut mir::LvalueProjection<'tcx>,
                      context: mir::visit::LvalueContext<'tcx>,
                      location: mir::Location) {
    if let mir::Lvalue::Local(local) = projection.base {
      if let mir::ProjectionElem::Field(field, ty) = projection.elem {
        let (ref target_string, index) = self.split_map[&local]
          .field_map[&field.index()];
        for target_node in self.tcx
          .hir
          .nodes_matching_suffix(&[target_string.clone()]) {
          if let ty::TypeVariants::TyAdt(adt, _) = self.tcx
            .item_type(self.tcx.hir.local_def_id(target_node))
            .sty {
            let target_local = self.decl_map[&local][adt];
            projection.base = mir::Lvalue::Local(target_local);
            projection.elem =
              mir::ProjectionElem::Field(mir::Field::new(index), ty);
          }
        }
      }
    }
    // self.super_projection(projection, context, location);
  }
}

#[derive(new)]
struct LvalueReplacer<'a> {
  local_index: usize,
  local_map: &'a HashMap<mir::Local, Vec<mir::Local>>,
}

impl<'tcx, 'a> visit::MutVisitor<'tcx> for LvalueReplacer<'a> {
  fn visit_lvalue(&mut self,
                  lvalue: &mut mir::Lvalue<'tcx>,
                  _: visit::LvalueContext<'tcx>,
                  _: mir::Location) {
    if let &mut mir::Lvalue::Local(local) = lvalue {
      if let Some(new_local_list) = self.local_map.get(&local) {
        *lvalue = mir::Lvalue::Local(new_local_list[self.local_index]);
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
    let local_index: HashMap<mir::Local, Vec<mir::Local>> = self.decl_maps
      .iter()
      .map(|(local, map)| {
        let mut local_vec = map.values().cloned().collect::<Vec<mir::Local>>();
        local_vec.sort();
        (*local, local_vec)
      })
      .collect();
    for statement in data.statements.clone().iter() {
      let location = mir::Location {
        block: block,
        statement_index: index,
      };
      index += 1;
      let mut visitors = vec![];
      let mut finder = LvalueFinder::new(self.decl_maps);
      finder.visit_statement(block, statement, location);
      if let Some(mir::Lvalue::Local(_)) = finder.value {
        // TODO: make this better
        for offset in 0..2 {
          visitors.push(LvalueReplacer::new(offset, &local_index));
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
