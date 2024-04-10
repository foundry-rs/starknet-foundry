# `parse_txt`


> `trait FileParser<T, impl TSerde: Serde<T>> {
>   fn parse_txt(file: @File) -> Option<T>;
> }`


## Accepted format
File content must consists of elements that:
- have to be separated with newlines
- have to be either:
  - integers in range of `[0, P)` where P is [`Cairo Prime`](https://book.cairo-lang.org/ch02-02-data-types.html?highlight=prime#felt-type) either in decimal or `0x` prefixed hex format
  - single line strings of length `<=31` (new lines can be used with \n)

For example, this file content:
```txt
1
2
hello
10
world
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
