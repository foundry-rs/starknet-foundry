# `parse_json`


> `trait FileParser<T, impl TSerde: Serde<T>> {
>fn parse_json(file: @File) -> Option<T>;
> }`

Parses json file content and tries to deserialize it to type `T` that implements `Serde` trait.

- `file` - a snapshot of an instance of the struct `File` that consists of the following fields:
    - `path` - Cairo shortstring representing a path to a file relative to a package root.

> ⚠️ **Warning**
>
>  A JSON object is an unordered data structure. To give it an order, the values in the array are sorted alphabetically by JSON keys.
To properly decode a JSON object, make sure the order of struct attributes aligns with the alphabetical order of the JSON keys.
>Nested JSON values are sorted by the flattened format keys `(a.b.c)`.

```rust
use option::OptionTrait;
use serde::Serde;
use snforge_std::io::{ FileTrait, FileParser };

#[derive(Serde, Drop)]
struct MyStruct {
    a: u32,
    b: felt252
}

#[test]
fn test_parse_json() {
    let file = FileTrait::new('data/file.json');
    let my_struct = FileParser::<MyStruct>::parse_json(@file).unwrap();
    // ...
}
```

File content must have proper JSON Format with values satisfying the conditions:
  - integers in range of `[0, P)` where P is [`Cairo Prime`](https://book.cairo-lang.org/ch02-02-data-types.html?highlight=prime#felt-type)
  - strings of length `<=31`
  - array of integers or strings fulfilling the above conditions

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
    a: 12,
    b: 1,
    c: "123"
    d: B {
        e: 1234
    }
}
```
