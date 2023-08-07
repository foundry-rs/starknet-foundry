# Loading data from files

## Context

Reading data from files and loading it into Cairo memory can be useful when testing
against different specified cases. This way users won't need to edit their Cairo code,
just edit the contents of their files.

Additionally, some major projects migrating to Starknet Foundry could benefit from
such a feature, as it was communicated by them.

## Goal

Propose a solution that will allow reading data from files to Cairo memory.

For example, the following test should pass when reading from a file with content like this:
```
2 90
```

```
let array = array![2, 90];
let file_data = read_array_from_file('file.txt').unwrap();
assert(*array[0] == *file_data[0] && *array[1] == *file_data[1], 'arrays are not equal');
```

## Proposed Solution

Create traits that are responsible for reading data from a file and loading it into Cairo memory
as an array of felts, returning a Result if something goes wrong (file does not exist, short string is too long, etc.).
Then the user can deserialize it themselves to the desired structure 
as described in [Cairo Book](https://book.cairo-lang.org/appendix-03-derivable-traits.html#serializing-with-serde).

### Supported type of files

For now, two types of files would be supported (in the future we can support also CSV, TOML, etc.):
- plain text files with space-separated felts like this
```
1 40 'hello' 100
```
- JSON files — order of elements in the output array would be as if traversing JSON tree with DFS
```json
{
  "a": 1,
  "b": {
    "c": {
        "array": [40, {"e": "hello"}]
      }
    },
  "d": 100
}
```
Note that short strings have to be in double quotes here due to JSON grammar definition.

### User interface
```
// struct to store path of the file
struct File {
    path: felt252 // relative path to the file
}

trait FileTrait {
    fn new(path: felt252) -> File;
}

impl FileTraitImpl of FileTrait {
    fn new(path: felt252) -> File {
        File { path }
    }
}

trait JsonParser {
    fn from_json(self: @File) -> Result<Array<felt252>, felt252>
}

trait TxtParser {
    fn from_txt(self: @File) -> Result<Array<felt252>, felt252>;
}

impl TxtParserImpl of JsonParser {
    fn from_txt(self: @File) -> Result<Array<felt252>, felt252> {
        // ...
    }
}

impl JsonParserImpl of TxtParser {
    fn from_txt(self: @File) -> Result<Array<felt252>, felt252> {
        // ...
    }
}
```

Example usage:
```
#[derive(Serde, Drop, PartialEq)]
struct A {
    item_one: felt252,
    item_two: felt252,
}

let file = File::new('data/file.json');
let data = file.from_json().unwrap();

let mut span = data.span();    
let deserialized_struct: A = Serde::<A>::deserialize(ref span).unwrap();
```
