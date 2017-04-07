#![allow(dead_code)]
#![feature(plugin, custom_derive)]
#![plugin(compiler)]
#[affinity_groups(a = 1, b = 2, c = 1)]
#[derive(Debug)]
#[derive(Clone)]
struct Test {
  pub a: usize,
  pub b: usize,
  pub c: usize,
}

impl Copy for Test {}

#[allow(unused_variables)]
fn main() {
  let mut y = [Test { a: 0, b: 0, c: 0 }; 10000];
  for i in 0..10000 {
    y[i].a = i;
    y[i].b = i + 1;
  }
  for i in 0..10000 {
    y[i].a += y[i].a;
    y[i].c += y[i].c;
  }
  for i in 0..10000 {
    y[i].b += y[i].b
  }
  for i in 0..10000 {
    println!{"{:?} {:?} {:?}", y[i].a, y[i].b, y[i].c};
  }
}
