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

    let path = ".a";
    let x = 1;
    let setter = Test::dynamic_setter::<i8>(path, x).unwrap();
    assert_eq!(format!("{}", path), path.to_string());
    test.set(setter).unwrap();
    let getter = Test::dynamic_getter(path).unwrap();
    assert_eq!(format!("{}", getter), path.to_string());
    let value = test.get(getter).unwrap();
    assert_eq!(value.parse_value::<i8>(path).unwrap(), x);

    let path = ".b.a";
    let x = 2;
    let setter = Test::dynamic_setter::<i8>(path, x).unwrap();
    assert_eq!(format!("{}", path), path.to_string());
    test.set(setter).unwrap();
    let getter = Test::dynamic_getter(path).unwrap();
    assert_eq!(format!("{}", getter), path.to_string());
    let value = test.get(getter).unwrap();
    assert_eq!(value.parse_value::<i8>(path).unwrap(), x);

    let path = ".c::B.a";
    let x = 3;
    let setter = Test::dynamic_setter::<i8>(path, x).unwrap();
    assert_eq!(format!("{}", path), path.to_string());
    test.set(setter).unwrap();
    let getter = Test::dynamic_getter(path).unwrap();
    assert_eq!(format!("{}", getter), path.to_string());
    let value = test.get(getter).unwrap();
    assert_eq!(value.parse_value::<i8>(path).unwrap(), x);

    let setter = Test::dynamic_setter::<()>(".c::A", ()).unwrap();
    test.set(setter).unwrap();
    match test.c {
        TestEnum::B(_) => {
            unreachable!();
        }
        TestEnum::A => {}
    }

    let path = ".d[4]";
    let x = 4;
    let setter = Test::dynamic_setter::<i8>(path, x).unwrap();
    assert_eq!(format!("{}", path), path.to_string());
    test.set(setter).unwrap();
    let getter = Test::dynamic_getter(path).unwrap();
    assert_eq!(format!("{}", getter), path.to_string());
    let value = test.get(getter).unwrap();
    assert_eq!(value.parse_value::<i8>(path).unwrap(), x);

    assert_eq!(Some(FieldsType::Fields(&[".a", ".b", ".c", ".d"])), <Test as RemoteGet>::GetterType::get_fields(""));
    assert_eq!(Some(FieldsType::Fields(&["::B", "VARIANT"])), <Test as RemoteGet>::GetterType::get_fields(".c"));
    assert_eq!(Some(FieldsType::Fields(&[".a", ".b"])), <Test as RemoteGet>::GetterType::get_fields(".c::B"));
    assert_eq!(Some(FieldsType::Terminal), <Test as RemoteGet>::GetterType::get_fields(".c::B.a"));
    assert_eq!(Some(FieldsType::Arr(8)), <Test as RemoteGet>::GetterType::get_fields(".d"));
}