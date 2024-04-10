# `parse_json`

> `trait FileParser<T, impl TSerde: Serde<T>> {
>   fn parse_json(file: @File) -> Option<T>;
> }`

> ⚠️ **Warning**
>
>  A JSON object is an unordered data structure. To give it an order, the values in the array are sorted alphabetically by JSON keys.
To properly decode a JSON object, make sure the order of struct attributes aligns with the alphabetical order of the JSON keys.
> Nested JSON values are sorted by the flattened format keys `(a.b.c)`.

## Accepted format
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
