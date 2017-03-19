#![feature(plugin, custom_derive)]
#![plugin(compiler)]
#![allow(dead_code)]
#[affinity_groups(a = 1, b = 2, c = 1)]
struct Test {
  pub a: i32,
  pub b: i32,
  pub c: i64,
}

#[allow(unused_variables)]
fn main() {
  let t = Test { b: 0, a: 0, c: 0 };
  let mut x = Test { a: 0, b: 0, c: 0 };
  x.a = 1;
  x.b = 2;
  x.c = 3;
  let y = [t, x];
  println!{"{}", y[1].a}
}
