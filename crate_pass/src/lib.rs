#![feature(plugin_registrar, rustc_private)]
#![feature(box_syntax)]

#[macro_use]
extern crate rustc;
extern crate rustc_plugin;
extern crate rustc_const_math;
extern crate syntax;

use rustc::mir::transform::{self, MirPass, MirSource};
use rustc::mir::repr::{Mir, Literal};
use rustc::mir::visit::MutVisitor;
use rustc::ty::TyCtxt;
use rustc::middle::const_val::ConstVal;
use rustc_const_math::ConstInt;
use rustc_plugin::Registry;

struct Pass;

impl transform::Pass for Pass {}

impl<'tcx> MirPass<'tcx> for Pass {
    fn run_pass<'a>(&mut self, _: TyCtxt<'a, 'tcx, 'tcx>, _: MirSource, mir: &mut Mir<'tcx>) {
        Visitor.visit_mir(mir)
    }
}

struct Visitor;

impl<'tcx> MutVisitor<'tcx> for Visitor {
    fn visit_literal(&mut self, literal: &mut Literal<'tcx>) {
        if let Literal::Value { ref mut value } = *literal {
            if let ConstVal::Integral(ConstInt::I32(ref mut i @ 11)) = *value {
                println!("Hello");
                *i = 42;
            }
        }
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_mir_pass(box Pass);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        println!("Hello World");
    }
}
