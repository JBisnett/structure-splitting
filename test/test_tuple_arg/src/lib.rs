#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}

#[derive(Clone)]
struct Test {
    a: u64,
    b: u64,
    c: u64,
}

struct Test1 {
    a: u64,
    c: u64,
}
struct Test2 {
    b: u64,
}

// fn test(tup: (&Test1, &Test2)) -> (Test1, Test2) {
//     let a = *tup.0;
//     let b = *tup.1;
//     return (a, b);
// }

// fn test2(ar: &Test1, br: &Test2) -> (Test1, Test2) {
//     let a = *ar;
//     let b = *br;
//     return (a, b);
// }

// fn test3(ar: &Test1) -> Test1 {
//     let a = *ar;
//     return a;
// }

// fn test4(ar: &Test) -> Test {
//     let a = *ar;
//     return a;
// }

fn test5(tup: (Test1, Test2)) -> (Test1, Test2) {
    let a = tup.0;
    let b = tup.1;
    return (a, b);
}

fn test6(t: Test1, v: Test2) {
    let (a, b) = test5((t, v));
    let c = a;
    let d = b;
}
