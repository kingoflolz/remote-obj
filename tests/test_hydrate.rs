use remote_obj::prelude::*;

#[derive(RemoteSetter, RemoteGetter)]
pub struct Test {
    a: i8,
    b: i8,
}

#[derive(RemoteSetter, RemoteGetter)]
pub struct NestedTest {
    a: Test,
    b: Test,
}

#[test]
fn test_hydrate() {
    let mut nested = NestedTest {
        a: Test {
            a: 0,
            b: 0
        },
        b: Test {
            a: 0,
            b: 0
        }
    };

    nested.set(setter!(NestedTest.a.a = 1)).unwrap();
    let g = getter!(NestedTest.a.a);
    let v = nested.get(g).unwrap();
    // assert_eq!(v.a().a(), 1);

    let mut buf = [0, 0, 0, 0];
    let raw_v = v.dehydrate(&mut buf).unwrap();
    assert_eq!(buf[0], 1);
    assert_eq!(raw_v, 1);

    let g = getter!(NestedTest.a.a);
    let (rehydrated_v, length) = <NestedTest as RemoteGet>::hydrate(g, &buf).unwrap();
    assert_eq!(length, 1);
    assert_eq!(rehydrated_v.a().a(), 1);
}