#![feature(quote, rustc_private)]
#![feature(box_syntax)]
#![feature(associated_consts)]
#![feature(box_patterns)]
#![feature(box_syntax)]
#![feature(conservative_impl_trait)]
#![feature(const_fn)]
#![cfg_attr(stage0,feature(field_init_shorthand))]
#![feature(i128_type)]
#![feature(loop_break_value)]
#![feature(quote)]
#![feature(rustc_diagnostic_macros)]
#![feature(rustc_private)]
#![feature(slice_patterns)]
#![feature(specialization)]
#![feature(staged_api)]
#![feature(unboxed_closures)]

extern crate rustc;
extern crate rustc_mir;
extern crate rustc_data_structures;
extern crate rustc_plugin;
extern crate rustc_const_math;
extern crate syntax;

#[macro_use]
extern crate derive_new;


pub mod lvalue_splitter;
pub mod struct_base_replacer;
pub mod split_struct;
pub mod deaggregator;
pub mod walkmut;
pub mod split_function_def;
pub mod split_function_call;
pub mod factor_function_call;
mod split_utils;
