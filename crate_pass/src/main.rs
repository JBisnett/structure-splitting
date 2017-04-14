#![allow(dead_code)]
#![feature(test, plugin, custom_derive)]
#![plugin(compiler)]
#[affinity_groups(a = 1, b = 2, c = 1)]
#[derive(Debug)]
#[derive(Clone)]
struct S {
  pub a: usize,
  pub b: usize,
  pub c: usize,
}
impl Copy for S {}

#[derive(Debug)]
#[derive(Clone)]
struct T {
  pub a: usize,
  pub b: usize,
  pub c: usize,
}
impl Copy for T {}

extern crate test;
#[test]
fn test_function_name() {
  assert_eq!(simple_test_s(), simple_test_t())
}

#[bench]
fn yes_pass(b: &mut test::Bencher) {
  b.iter(|| { simple_test_s(); })
}

#[bench]
fn no_pass(b: &mut test::Bencher) {
  b.iter(|| { simple_test_t(); })
}

fn simple_test_s() -> usize {
  let mut y = [S { a: 0, b: 0, c: 0 }; 100000];
  let mut sum = 0;
  for i in 0..100000 {
    y[i].a = i;
    y[i].b = i + 1;
  }
  for i in 0..100000 {
    y[i].a += y[i].a;
    y[i].c += y[i].c;
  }
  for i in 0..100000 {
    y[i].b += y[i].b
  }
  for i in 0..100000 {
    sum += y[i].a;
    sum += y[i].b;
    sum += y[i].c;
  }
  sum
}

fn simple_test_t() -> usize {
  let mut y = [T { a: 0, b: 0, c: 0 }; 100000];
  for i in 0..100000 {
    y[i].a = i;
    y[i].b = i + 1;
  }
  for i in 0..100000 {
    y[i].a += y[i].a;
    y[i].c += y[i].c;
  }
  for i in 0..100000 {
    y[i].b += y[i].b
  }
  let mut sum = 0;
  for i in 0..100000 {
    sum += y[i].a;
    sum += y[i].b;
    sum += y[i].c;
  }
  sum
}

#[allow(unused_variables)]
fn main() {
  println!{"{}", simple_test_s()};
}
