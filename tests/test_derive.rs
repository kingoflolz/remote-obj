use remote_obj::prelude::*;

#[derive(RemoteSetter, RemoteGetter)]
pub struct Test {
    a: i8,
    b: i8,
}

#[derive(RemoteSetter, RemoteGetter)]
pub struct NewType {
    a: i8,
    b: i8,
}

#[derive(RemoteSetter, RemoteGetter)]
pub enum TestEnum {
    A,
    B(NewType)
}

#[derive(RemoteSetter, RemoteGetter)]
pub struct Config<'a> {
    a: &'a mut i8,
    #[remote(read_only)]
    b: i8,
    #[remote(write_only)]
    c: i8,
    d: Test,
    e: TestEnum,
    f: [i8; 8]
}

#[test]
fn test_derive() {
    let mut a = 0;
    let new = NewType {
        a: 0,
        b: 0
    };
    let mut config = Config {
        a: &mut a,
        b: 0,
        c: 0,
        d: Test {
            a: 0,
            b: 0
        },
        e: TestEnum::B(new),
        f: [0; 8]
    };
    // read-only
    // config.set(setter!(Config.b(1)));

    // write-only
    // let v = config.get(getter!(Config.c())).unwrap();
    // assert_eq!(v.c(), 0);

    // field set
    config.set(setter!(Config.a = 1)).unwrap();

    // field get
    let v = config.get(getter!(Config.a)).unwrap();
    assert_eq!(v.a(), 1);

    // nested field set
    config.set(setter!(Config.d.a = 2)).unwrap();

    // nested field get
    let v = config.get(getter!(Config.d.a)).unwrap();
    assert_eq!(v.d().a(), 2);

    // enum variant set inner field
    config.set(setter!(Config.e::B.a = 3)).unwrap();

    // enum variant get inner field
    let v = config.get(getter!(Config.e::B.a)).unwrap();
    assert_eq!(v.e().B().a(), 3);

    // enum set variant
    config.set(setter!(Config.e::A)).unwrap();

    // check that enum doesn't allow write to inner if variant is incorrect
    assert!(config.set(setter!(Config.e::B.a = 2)).is_err());

    // enum get variant
    let v = config.get(getter!(Config.e.var)).unwrap();
    match v.e() {
        <TestEnum as Getter>::ValueType::A => {},
        _ => unreachable!(),
    }

    // array field set
    config.set(setter!(Config.f[1] = 1)).unwrap();

    // array field get
    let v = config.get(getter!(Config.f[1])).unwrap();
    assert_eq!(v.f()[1], 1);
}