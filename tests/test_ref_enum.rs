use remote_obj::prelude::*;

#[derive(RemoteSetter, RemoteGetter)]
pub struct Test<'a> {
    a: &'a mut i8,
    b: &'a mut i8,
}

#[derive(RemoteSetter, RemoteGetter)]
pub enum EnumTest<'a> {
    A(&'a mut Test<'a>),
    B(&'a mut Test<'a>),
}

#[test]
fn test_ref_enum() {
    let mut a = 0;
    let mut b = 0;
    let mut test_ab = Test {
        a: &mut a,
        b: &mut b,
    };

    let mut nested = EnumTest::A(&mut test_ab);

    nested.set(setter!(EnumTest::A.a = 1)).unwrap();
    let v = nested.get(getter!(EnumTest::A.a)).unwrap();
    assert_eq!(v.A().a(), 1);
}