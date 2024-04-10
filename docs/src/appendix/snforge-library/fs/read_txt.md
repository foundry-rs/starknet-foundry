# `read_txt`

> `fn read_txt(file: @File) -> Array<felt252>`

Read plain text file content to an array of felts.

## Accepted format
File content must consists of elements that:
- have to be separated with newlines
- have to be either:
  - integers in range of `[0, P)` where P is [`Cairo Prime`](https://book.cairo-lang.org/ch02-02-data-types.html?highlight=prime#felt-type) either in decimal or `0x` prefixed hex format
  - single line short strings (`felt252`) of length `<=31` surrounded by `''` i.e. `'short string'`, new lines can be used with `\n` and `'` with `\'`
  - single line strings (`ByteArray`) surrounded by `""` i.e. `"very very very very loooooong string"`, new lines can be used with `\n` and `"` with `\"`

For example, this file content:
```txt
1
2
'hello'
10
'world'
```
will be parsed to the following array:
```rust
array![1, 2, 'hello', 10, 'world']
```
