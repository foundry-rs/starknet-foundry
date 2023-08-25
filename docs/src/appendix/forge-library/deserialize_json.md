# `deserialize_json`


> `trait Parser<T, impl TSerde: Serde<T>> {
>     fn deserialize_json(file: @File) -> Option<T>;
> }` 

Parses json file content and tries to deserialize it to type `T` that implements `Serde` trait.

- `file` - a snapshot of an instance of the struct `File` that consists of the following fields:
  - `path` - Cairo shortstring representing a path to a file relative to a package root.

> ⚠️ **Warning**
>
>  JSON object is an unordered data, we had to somehow give it order. Therefore, the values in the array are sorted alphabetically by JSON key. That means that in order to decode the JSON object correctly, you will need to define attributes of the struct with types that correspond to the values of the alphabetical order of the keys of the JSON.
```rust
use option::OptionTrait;
use serde::Serde;
use snforge_std::{ FileTrait, Parser };

#[derive(Serde, Drop)]
struct MyStruct {
    a: u32,
    b: felt252
}

#[test]
fn test_deserialize_json() {
    let file = FileTrait::new('data/file.json');
    let my_struct = Parser::<MyStruct>::deserialize_json(@file).unwrap();
    // ...
}
```

File content must have proper JSON Format with values satisfying the conditions:
  - integers in range of `[0, P)` where P is [`Cairo Prime`](https://book.cairo-lang.org/ch02-02-data-types.html?highlight=prime#felt-type)
  - strings of length `<=31`

For example, this file content:
```json
{
    "b": 1,
    "a": 12,
    "d": {
        "e": 1234
    },
    "c": "123"
}
```
could be parsed to the following struct:

```rust
A {
    ab: 12,
    bc: 1,
    cda: "123"
    d: B {
        e: 1234
    }
}
```
