# Remote-Obj

A rust proc-macro which allows for reading and writing fields/variants of (possibly nested)
remote objects by generating a single enum which covers all field reads and writes.

Intended for reading/writing config fields in embedded systems lazily, where bandwidth
might be limited (and hence reading/writing the whole struct using something like 
serde is not desirable).

## Examples
See `tests/test_derive.rs`