# `parse_txt`


> `trait FileParser<T, impl TSerde: Serde<T>> {
>fn parse_txt(file: @File) -> Option<T>;
> }`


Parses plain text file content and tries to deserialize it to type `T` that implements `Serde` trait.

- `file` - a snapshot of an instance of the struct `File` that consists of the following fields:
  - `path` - Cairo shortstring representing a path to a file relative to a package root.

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
fn test_parse_txt() {
    let file = FileTrait::new('data/file.txt');
    let my_struct = FileParser::<MyStruct>::parse_txt(@file).unwrap();
    // ...
}
```

File content must consists of elements that:
- have to be separated with whitespaces
- have to be either:
  - integers in range of `[0, P)` where P is [`Cairo Prime`](https://book.cairo-lang.org/ch02-02-data-types.html?highlight=prime#felt-type)
  - strings of length `<=31` enclosed in single quotation marks

For example, this file content:
```txt
1  2   'hello' 10     'world'
```
could be parsed to the following struct:
```rust
A {
    a: 1, 
    nested: B {
        b: 2,
        c: 'hello',
    }, 
    d: 10,
    e: 'world'
}
```
