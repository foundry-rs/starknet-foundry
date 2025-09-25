# File Format Rules

Some rules have to be checked when providing a file for snforge, in order for correct parsing behavior.
Different ones apply for JSON and plain text files.

## Plain Text Files
- Elements have to be separated with newlines
- Elements have to be either:
  - Cairo `felt252` values, surrounded by `''`
  - byte arrays, surrounded by `""`

### Example

Below is an example of valid plain text file:
```txt
1
2
'hello'
10
"world"
```

## JSON Files
- Elements have to be either:
  - Cairo `felt252` values, surrounded by `''`
  - byte arrays, surrounded by `""`
  - array of integers or strings fulfilling the above conditions

> ⚠️ **Warning**
>
> A JSON object is an unordered data structure. To make reading JSONs deterministic, the values are read from the JSON in an order that is alphabetical in respect to JSON keys.
> Nested JSON values are sorted by the flattened format keys `(a.b.c)`.

### Example

Below is an example of valid JSON file:
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
