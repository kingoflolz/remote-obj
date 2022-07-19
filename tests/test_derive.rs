use remote_obj::{RemoteSetter, RemoteGetter, setter, getter, Setter, Getter};

#[derive(RemoteSetter, RemoteGetter)]
pub struct Test {
    pub a: i8,
    pub b: i8,
}

impl Test {
    fn new() -> Self {
        Test {
            a: 0,
            b: 0,
        }
    }
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
pub struct Config {
    // #[remote(skip)]
    a: i8,
    // #[remote(read_only)]
    b: i8,
    c: i8,
    d: Test,
    e: TestEnum,
}

impl Config {
    fn new() -> Config {
        Config {
            a: 0,
            b: 0,
            c: 0,
            d: Test::new(),
            e: TestEnum::B(NewType {
                a: 0,
                b: 0,
            }),
        }
    }
}

#[test]
fn test_setter() {
    let mut config = Config::new();
    // skipped
    // config.set(ConfigSetter::a(1));

    // read-only
    // config.set(ConfigSetter::b(1));

    config.set(setter!(Config.c(1))).unwrap();
    config.set(setter!(Config.d.a(2))).unwrap();

    config.set(setter!(Config.e.B.a(2))).unwrap();

    let v = config.get(getter!(Config.e.B.a(()))).unwrap();
    assert_eq!(v.e().B().a(), 2);

    config.set(setter!(Config.e.A(()))).unwrap();
    assert!(config.set(setter!(Config.e.B.a(2))).is_err());

    let v = config.get(getter!(Config.e.var())).unwrap();
    match v.e() {
        <TestEnum as Getter>::ValueType::A => {},
        _ => unreachable!(),
    }

    let v = config.get(getter!(Config.c())).unwrap();
    assert_eq!(v.c(), 1);

    let v = config.get(getter!(Config.d.a())).unwrap();
    assert_eq!(v.d().a(), 2);
}