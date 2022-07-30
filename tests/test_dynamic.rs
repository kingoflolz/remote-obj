use remote_obj::prelude::*;

#[derive(RemoteSetter, RemoteGetter)]
pub struct Nested {
    a: i8,
    b: i8,
}

#[derive(RemoteSetter, RemoteGetter)]
pub struct Test {
    a: i8,
    b: Nested,
    c: TestEnum,
    d: [i8; 8],
}

#[derive(RemoteSetter, RemoteGetter)]
pub enum TestEnum {
    A,
    B(Nested),
}

#[test]
fn test_dynamic() {
    let mut test = Test {
        a: 0,
        b: Nested {
            a: 0,
            b: 0
        },
        c: TestEnum::B(
            Nested {
                a: 0,
                b: 0
            }
        ),
        d: [0; 8]
    };

    let setter = Test::dynamic_setter::<i8>(".a", 1).unwrap();
    test.set(setter).unwrap();
    let getter = Test::dynamic_getter(".a").unwrap();
    let value = test.get(getter).unwrap();
    assert_eq!(value.parse_value::<i8>(".a").unwrap(), 1);

    let setter = Test::dynamic_setter::<i8>(".b.a", 2).unwrap();
    test.set(setter).unwrap();
    let getter = Test::dynamic_getter(".b.a").unwrap();
    let value = test.get(getter).unwrap();
    assert_eq!(value.parse_value::<i8>(".b.a").unwrap(), 2);

    let setter = Test::dynamic_setter::<i8>(".c::B.a", 3).unwrap();
    test.set(setter).unwrap();
    let getter = Test::dynamic_getter(".c::B.a").unwrap();
    let value = test.get(getter).unwrap();
    assert_eq!(value.parse_value::<i8>(".c::B.a").unwrap(), 3);

    let setter = Test::dynamic_setter::<()>(".c::A", ()).unwrap();
    test.set(setter).unwrap();
    match test.c {
        TestEnum::B(_) => {
            unreachable!();
        }
        TestEnum::A => {}
    }

    let setter = Test::dynamic_setter::<i8>(".d[4]", 4).unwrap();
    test.set(setter).unwrap();
    let getter = Test::dynamic_getter(".d[4]").unwrap();
    let value = test.get(getter).unwrap();
    assert_eq!(value.parse_value::<i8>(".d[4]").unwrap(), 4);
}