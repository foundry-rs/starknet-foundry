# `fs` Module

Module containing functions for interacting with the filesystem.

## File format
Some rules have to be checked when providing a file for snforge, in order for correct parsing behavior.
Different ones apply for JSON and plain text files.

### Plain text files
- Elements have to be separated with newlines
- Elements have to be either:
  - integers in range of `[0, P)` where P is [`Cairo Prime`](https://book.cairo-lang.org/ch02-02-data-types.html?highlight=prime#felt-type) either in decimal or `0x` prefixed hex format
  - single line short strings (`felt252`) of length `<=31` surrounded by `''` i.e., `'short string'`, new lines can be used with `\n` and `'` with `\'`
  - single line strings (`ByteArray`) surrounded by `""` i.e., `"very very very very loooooong string"`, new lines can be used with `\n` and `"` with `\"`


### JSON files
- Elements have to be either:
  - integers in range of `[0, P)` where P is [`Cairo Prime`](https://book.cairo-lang.org/ch02-02-data-types.html?highlight=prime#felt-type)
  - single line strings (`ByteArray`) i.e. `"very very very very loooooong string"`, new lines can be used with `\n` and `"` with `\"`
  - array of integers or strings fulfilling the above conditions

> ⚠️ **Warning**
>
> A JSON object is an unordered data structure. To make reading JSONs deterministic, the values are read from the JSON in an order that is alphabetical in respect to JSON keys.
> Nested JSON values are sorted by the flattened format keys `(a.b.c)`.


## Example
 
For example, this plain text file content:
```txt
1
2
'hello'
10
"world"
```
or this JSON file content:
```json
{
  "a": 1,
  "nested": {
    "b": 2,
    "c": 448378203247
  },
  "d": 10,
  "e": "world"
}
``` 

(note that short strings cannot be used in JSON file)

could be parsed to the following struct in cairo, via `parse_txt`/`parse_json`:
```rust
A {
    a: 1, 
    nested: B {
        b: 2,
        c: 'hello',
    }, 
    d: 10,
    e: "world"
}
```

or to an array, via `read_txt`/`read_json`:
```rust
array![1, 2, 'hello', 10, 0, 512970878052, 5]
```