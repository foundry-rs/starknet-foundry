# File format rules

Some rules have to be checked when providing a file for snforge, in order for correct parsing behavior.
Different ones apply for JSON and plain text files.

## Plain text files
- Elements have to be separated with newlines
- Elements have to be either:
  - Cairo `felt252` values, surrounded by `''`
  - byte arrays, surrounded by `""`

## JSON files
- Elements have to be either:
  - Cairo `felt252` values, surrounded by `''`
  - byte arrays, surrounded by `""`
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