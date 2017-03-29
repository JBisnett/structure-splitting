#![allow(dead_code)]
#![feature(plugin, custom_derive)]
#![plugin(compiler)]
#[affinity_groups(a = 1, b = 2, c = 1)]
// #[derive(Debug)]
struct Test {
  pub a: i32,
  pub b: i32,
  pub c: i64,
}

fn test_print(x: i32) {
  println!{"{}", x};
}

#[allow(unused_variables)]
fn main() {
  let t = Test { b: 0, a: 0, c: 0 };
  let mut x = Test { a: 0, b: 0, c: 0 };
  x.a = 1;
  x.b = 2;
  x.c = 3;
  let y = [t, x];
  test_print(y[1].a);
}
