use rustc::ty::TyCtxt;
use rustc::mir;
use rustc::mir::visit;
use rustc::mir::visit::Visitor;

use split_struct::{LocalMap};

use std::collections::hash_map::HashMap;

#[derive(new)]
struct LvalueMultiReplacer<'a> {
  local_index: usize,
  local_map: &'a HashMap<mir::Local, Vec<mir::Local>>,
}

impl<'tcx, 'a> visit::MutVisitor<'tcx> for LvalueMultiReplacer<'a> {
  fn visit_lvalue(&mut self,
                  lvalue: &mut mir::Lvalue<'tcx>,
                  context: visit::LvalueContext<'tcx>,
                  location: mir::Location) {
    if let &mut mir::Lvalue::Local(local) = lvalue {
      if let Some(new_local_list) = self.local_map.get(&local) {
        *lvalue = mir::Lvalue::Local(new_local_list[self.local_index]);
      }
    } else {
      self.super_lvalue(lvalue, context, location);
    }
  }
}
struct LvalueFinder<'tcx> {
  decl_maps: &'tcx LocalMap<'tcx>,
  value: Option<mir::Lvalue<'tcx>>,
}

impl<'tcx> LvalueFinder<'tcx> {
  fn new(decl_maps: &'tcx LocalMap<'tcx>) -> Self {
    LvalueFinder {
      decl_maps: decl_maps,
      value: None,
    }
  }
}

impl<'tcx> visit::Visitor<'tcx> for LvalueFinder<'tcx> {
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
pub struct StructLvalueSplitter<'a, 'tcx: 'a + 'v, 'v> {
  tcx: TyCtxt<'a, 'tcx, 'tcx>,
  mir: &'v mir::Mir<'tcx>,
  decl_maps: &'a LocalMap<'tcx>,
  // split_map: &'a SplitMap<'tcx>,
}

impl<'a, 'tcx, 'v> visit::MutVisitor<'tcx>
  for StructLvalueSplitter<'a, 'tcx, 'v> {
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
      // if let mir::StatementKind::Assign(lv, rv) = statement.kind {
      //   let assign_ty = lv.ty(self.mir, self.tcx).to_ty(self.tcx);
      //   if let Some(split_struct) = split_map.get(assign_ty); for offset in 0..split_struct.child_names.length {visitors.push(LvalueMultiReplacer::new(offset, &local_index));}}
      let mut finder = LvalueFinder::new(self.decl_maps);
      finder.visit_statement(block, statement, location);
      if let Some(mir::Lvalue::Local(_)) = finder.value {
        // FIXME: this does not work for >2 substructs
        for offset in 0..2 {
          visitors.push(LvalueMultiReplacer::new(offset, &local_index));
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

  fn visit_rvalue(&mut self,
                  rvalue: &mut mir::Rvalue<'tcx>,
                  location: mir::Location) {
    self.super_rvalue(rvalue, location);
    if let mir::Rvalue::Aggregate(mir::AggregateKind::Array(_), ref ops) =
           rvalue.clone() {
      if let Some(first_op) = ops.get(0) {
        *rvalue =
          mir::Rvalue::Aggregate(mir::AggregateKind::Array(
            first_op.ty(&self.mir, self.tcx)), ops.clone());
      }
    }
  }
}
