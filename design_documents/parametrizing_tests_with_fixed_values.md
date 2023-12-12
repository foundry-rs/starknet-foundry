# Parametrizing Tests With Fixed Values

## Context

Various test runners for other programming languages provide ways of parametrizing test cases with fixed values.
This way, multiple test cases can be executed for the same test source code.

## Goal

Propose a way of adding support for parametrized tests to Starknet Foundry's `snforge`.

## Proposed Solution

Introduce three new attributes: `parametrize`, `case` and.

- `parametrize` - defines a parametrized test.
- `case` - defines a specific test case

### Parametrization With Simple Arguments

```cairo
#[parametrize]
#[case(1, 3)]
#[case(3, 5)]
#[test]
fn my_test(a: felt252, b: u32) {
    // case a = 1, b = 3
    // case a = 3, b = 5
    // ...
}
```

For test cases like this, values could simply be injected into a runner for each `case`.

### Parametrization With Fixtures

```cairo
struct MyStruct {
    a: felt252, 
    b: Array<felt252>
}

fn my_fixture() -> MyStruct {
    MyStruct { a: 1, b: array![2, 3] }
}

#[parametrize]
#[case(1, my_fixture())]
#[case(3, my_fixture())]
#[test]
fn my_test(a: felt252, b: MyStruct) {
    // case a = 1, b = MyStruct { a: 1, b: [2, 3] }
    // case a = 3, b = MyStruct { a: 1, b: [2, 3] }
    // ...
}

// This would require some code generation
```

To handle parametrization with function calls, we would have to generate a necessary code.
An example generated code for the snippet above could look like this

```cairo
// ...
#[test]
fn my_test_generated(a: felt252) {
    let b = my_fixture();
    // ...
}
```

### Parametrized Fixtures

```cairo
struct MyStruct {
    a: felt252, 
    b: Array<felt252>
}

fn my_fixture(a: felt252, b: felt252, c: felt252) -> MyStruct {
    MyStruct { a: a, b: array![b, c] }
}

#[parametrize]
#[case(1, my_fixture(2, 3, 4))]
#[case(3, my_fixture(5, 6, 7))]
#[test]
fn my_test(a: felt252, b: MyStruct) {
    // case a = 1, b = MyStruct { a: 2, b: [3, 4] }
    // case a = 3, b = MyStruct { a: 5, b: [6, 7] }
    // ...
}

// This would require some code generation
```

This could be handled in a very similar manner as the [non-parametrized fixture case](#parametrization-with-fixtures):

```cairo
// ...
#[test]
fn my_test_generated(a: felt252) {
    let b = my_fixture(2, 3, 4);
    // ...
}
```
