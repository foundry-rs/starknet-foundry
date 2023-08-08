# Debugging

> ℹ️ **Info**
> To use `PrintTrait` you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies#adding-a-dependency) 
> using appropriate release tag.
>```toml
> [dependencies]
> snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.4.0" }
> ```

Starknet Foundry standard library adds a utility method for printing inside tests to facilitate simple debugging.

Here's a test with example use of print method:

```rust
// Make sure to import Starknet Foundry PrintTrait
use array::ArrayTrait;
use snforge_std::PrintTrait;

#[test]
fn test_print() {
    'Print short string:'.print();
    'my string'.print();

    'Print number:'.print();
    123.print();

    'Print array:'.print();
    let mut arr = ArrayTrait::new();
    arr.append(1);
    arr.append(2);
    arr.append(3);
    arr.print();

    'Print bool:'.print();
    (1 == 5).print();
    assert(1 == 1, 'simple check');
}
```

Running tests will include prints in the output:

```shell
$ snforge
Collected 1 test(s) and 2 test file(s)
Running 0 test(s) from package_name package
Running 1 test(s) from tests/test_print.cairo
original value: [1794026292945241370577200538206453096157964090], converted to a string: [Print short string:]
original value: [2019423207056158060135], converted to a string: [my string]
original value: [6373661751074312243962081276474], converted to a string: [Print number:]
original value: [123], converted to a string: [{]
original value: [97254360215367257408299385], converted to a string: [Print array:]
original value: [1], converted to a string: []
original value: [2], converted to a string: []
original value: [3], converted to a string: []
original value: [379899844591278365831020], converted to a string: [Print bool:]
original value: [439721161573], converted to a string: [false]
[PASS] test_print::test_print
Tests: 1 passed, 0 failed, 0 skipped
```

Forge tries to convert values to strings when possible. In case conversion is not possible,
just `original value` is printed.

> ℹ️ **Info**
> More debugging features will be added to Starknet Foundry once Starknet compiler implements necessary support.
