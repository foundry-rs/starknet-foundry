# `read_txt`

> `fn read_txt(file: @File) -> Array<felt252>`

Read and parses plain text file content to an array of felts.

- `file` - a snapshot of an instance of the struct `File` that consists of the following fields:
  - `path` - Cairo shortstring representing a path to a file relative to a package root.

```rust
use snforge_std::io::{ FileTrait, read_txt };

#[test]
fn test_read_txt() {
    let file = FileTrait::new('data/file.txt');
    let content = read_txt(@file);
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
will be parsed to the following array:
```rust
array![1, 2, 'hello', 10, 'world']
```
