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
};

let x = SomeStruct::new();

// setter is a regular enum, and is the same type for any field
let setter = set!(SomeStruct.field_name = expression);

// can easily send setter over the wire/serialize it
x.set(setter).unwrap();

// works similarly with getters, the getter is the same type for any field
let getter = get!(SomeStruct.field_name);

// value is the same type for any field
let value = x.get(getter).unwrap();

// inner value actually returns the real value of the field
let inner_value = value.field_name();
```

## Examples
See `tests/test_derive.rs`