# `read_json`

> `fn read_json(file: @File) -> Array<felt252>`

> ⚠️ **Warning**
>
> A JSON object is an unordered data structure. To give it an order, the values in the array are sorted alphabetically by JSON keys. Therefore, the values in the array are sorted alphabetically by JSON key.
> Nested JSON values are sorted by the flattened format keys `(a.b.c)`.

## Accepted format
File content must have proper JSON Format with values satisfying the conditions:
  - integers in range of `[0, P)` where P is [`Cairo Prime`](https://book.cairo-lang.org/ch02-02-data-types.html?highlight=prime#felt-type)
  - single line strings (`ByteArray`) i.e. `"very very very very loooooong string"`, new lines can be used with `\n` and `"` with `\"`
  - array of integers or strings fulfilling the above conditions

For example, this file content:
```json
{
    "b": 1,
    "a": 12,
    "d": {
        "e": 1234
    },
    "c": 123
}
```
will be read to the following array:

```rust
array![12, 1, 123, 1234]
```
