#![allow(dead_code)]
#![feature(test, plugin, custom_derive)]
#![plugin(compiler)]

use std::iter::FromIterator;

struct VecTypeContainer(Vec<S>);

#[affinity_groups(a = 1, b = 2, c = 1)]
#[derive(Debug)]
#[derive(Clone)]
struct S {
	pub a: usize,
	pub b: usize,
	pub c: usize,
}
struct Stup(S1, S2);
struct SRtup<'a>(&'a S1, &'a S2);
struct SRGtup<'a, A, B>(&'a A, &'a B);
impl<'a> AsRef<SRtup<'a>> for SRtup<'a> {
	fn as_ref(&self) -> &Self {
		&self
	}
}
impl<'a, A, B> AsRef<SRGtup<'a, A, B>> for SRtup<'a, A, B> {
	fn as_ref(&self) -> &Self {
		&self
	}
}
struct SVtup(Vec<S1>, Vec<S2>);
impl SVtup {
	fn push(&mut self, Stup(s1, s2): Stup) {
		self.0.push(s1);
		self.1.push(s2);
	}
}
impl FromIterator<Stup> for SVtup {
	fn from_iter<I: IntoIterator<Item = Stup>>(iter: I) -> Self {
		let mut sv = SVtup(vec![], vec![]);
		for i in iter {
			sv.push(i);
		}
		sv
	}
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
	let test = Vec::new();
	test.push(S { a: 0, b: 0, c: 0 });
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
	let test = vec![T { a: 0, b: 0, c: 0 }];
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

fn main() {
	println!{"{}", simple_test_s()};
}
