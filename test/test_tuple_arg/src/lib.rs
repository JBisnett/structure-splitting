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
struct TestT(Test1, Test2);
struct TestR<'a>(&'a Test1, &'a Test2);


impl<'a> AsRef<TestR<'a>> for TestR<'a> {
    fn as_ref(&self) -> &TestR<'a> {
        return self;
    }
}

impl<'a> AsMut<TestR<'a>> for TestR<'a> {
    fn as_mut(&mut self) -> &mut TestR<'a> {
        return self;
    }
}
fn test() {}
