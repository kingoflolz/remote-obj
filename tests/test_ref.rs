use remote_obj::prelude::*;

#[derive(RemoteSetter, RemoteGetter)]
pub struct TestInner {
    a: i8,
    b: i8,
}

#[derive(RemoteSetter, RemoteGetter)]
pub enum EnumTest {
    A(TestInner),
    B,
}

#[derive(RemoteSetter, RemoteGetter)]
pub struct Test<'a> {
    a: &'a mut i8,
    b: &'a mut i8,
    c: &'a mut EnumTest,
}

#[test]
fn test_ref() {
    let mut a = 0;
    let mut b = 0;
    let mut test = Test {
        a: &mut a,
        b: &mut b,
        c: &mut EnumTest::A(TestInner{
            a: 0,
            b: 0
        })
    };

    test.set(setter!(Test.a = 1)).unwrap();
    let v = test.get(getter!(Test.a)).unwrap();
    assert_eq!(v.a(), 1);

    test.set(setter!(Test.c::A.a = 1)).unwrap();
    let v = test.get(getter!(Test.c::A.a)).unwrap();
    assert_eq!(v.c().A().a(), 1);
}