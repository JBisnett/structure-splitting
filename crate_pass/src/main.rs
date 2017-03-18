#![feature(plugin, custom_derive)]
#![plugin(compiler)]
#![allow(dead_code)]
#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;

#[affinity_groups(tester1 = 1, tester2 = 2, tester3 = 1)]
#[derive(Debug)]
struct Test {
  pub tester1: i32,
  pub tester2: i32,
  pub tester3: i64,
}

// FIXME newtype the usize

trait TypeStack<T> {
  fn insert(&mut self, x: T) -> usize;
  fn get(&self, usize) -> &T;
  fn remove(&mut self, usize);
}

struct SimpleStack<T> {
  heap: Vec<T>,
}

impl<T> TypeStack<T> for SimpleStack<T> {
  fn insert(&mut self, x: T) -> usize {
    let val = self.heap.len();
    self.heap.push(x);
    val
  }
  fn remove(&mut self, index: usize) {
    self.heap.remove(index);
  }
  fn get(&self, index: usize) -> &T {
    &self.heap[index]
  }
}

lazy_static! {
  static ref TEST_HEAP : Mutex<Vec<Test>> =
    Mutex::new(vec![]);
}

#[allow(unused_variables)]
fn main() {
  let t = Test {
    tester2: 0,
    tester1: 0,
    tester3: 0,
  };
  let mut x = Test {
    tester1: 0,
    tester2: 0,
    tester3: 0,
  };
  x.tester1 = 1;
  x.tester2 = 2;
  x.tester3 = 3;
  let f = [t, x];
  println!{"{:?}", f};
}
