use remote_obj::prelude::*;

#[derive(RemoteSetter, RemoteGetter)]
pub struct Test<'a> {
    a: &'a mut i8,
    b: &'a mut i8,
}

#[derive(RemoteSetter, RemoteGetter)]
pub struct NestedTest<'a> {
    a: &'a mut Test<'a>,
    b: &'a mut Test<'a>,
}

#[test]
fn test_ref() {
    let mut a = 0;
    let mut b = 0;
    let mut test_ab = Test {
        a: &mut a,
        b: &mut b,
    };

    let mut c = 0;
    let mut d = 0;
    let mut test_cd = Test {
        a: &mut c,
        b: &mut d,
    };

    let mut nested = NestedTest {
        a: &mut test_ab,
        b: &mut test_cd,
    };

    nested.set(setter!(NestedTest.a.a = 1)).unwrap();
    let v = nested.get(getter!(NestedTest.a.a)).unwrap();
    assert_eq!(v.a().a(), 1);
}