# Remote-Obj

A rust proc-macro which allows for reading and writing fields/variants of (possibly nested)
remote objects by generating a single enum which covers all field reads and writes.

Intended for reading/writing config fields in embedded systems lazily, where bandwidth
might be limited (and hence reading/writing the whole struct using something like 
serde is not desirable).

```rust
#[derive(RemoteSetter, RemoteGetter)]
struct SomeStruct {
    // ...
}

fn main() {
    let x = SomeStruct::new();

    // setter is a regular enum, and is the same type for any field
    let setter: <SomeStruct as Setter>::SetterType = set!(SomeStruct.field_name = expression);

    // can easily send setter over the wire/serialize it
    x.set(setter).unwrap();

    // works similarly with getters, the getter is the same type for any field
    let getter: <SomeStruct as Getter>::GetterType = get!(SomeStruct.field_name);

    // value is the same type for any field
    let value: <SomeStruct as Getter>::ValueType = x.get(getter).unwrap();

    // inner value actually returns the real value of the field
    let inner_value = value.field_name();
}
```

## Dehydration
Also supports a more efficient method for transferring ValueTypes if both sides can agree on the interpretation (such
as if the metadata is transferred beforehand, or if there is some sort of static mapping (such as CAN ids)).

```rust
let getter: <SomeStruct as Getter>::GetterType = get!(SomeStruct.field_name);
let value: <SomeStruct as Getter>::ValueType = x.get(getter).unwrap();

// here we take the ValueType and convert it to some raw bytes in a buffer
let length = value.dehydrate(&mut buf).unwrap();

// send the buffer over the wire

// on the other side, using the GetterType, we can hydrate the value from the buffer
let (rehydrated_v, same_length) = <SomeStruct as Getter>::hydrate(getter, &buf).unwrap();

assert_eq!(value, rehydrated_v);
assert_eq!(length, same_length);
```

## Examples
See `tests/test_derive.rs`