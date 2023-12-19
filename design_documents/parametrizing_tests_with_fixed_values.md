# Parametrizing Tests With Fixed Values

## Context

Various test runners for other programming languages provide ways of parametrizing test cases with fixed values.
This way, multiple test cases can be executed for the same test source code.

## Goal

Propose a way of adding support for parametrized tests to Starknet Foundry's `snforge`.

## Proposed Solution

Introduce two new attributes: `parametrize` and `case`.

- `parametrize` - defines a parametrized test.
- `case` - defines a specific test case

Generate code for each of the test cases so no handling of values injection, etc. is necessary.
The compiler would then validate if the values for the generated code are actually valid.
Thanks to that, we could have failures for invalid arguments early in the execution.

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

For test cases like this, we would generate two actual test cases:

```cairo
#[test]
fn my_test_case_1() {
  let a: felt252 = 3;
  let b: u32 = 1;
}

#[test]
fn my_test_case_1() {
  let a: felt252 = 3;
  let b: u32 = 5;
}
```

The explicit type annotations for each argument are necessary.
This way, if the user provides invalid value for the type defined, the compiler will throw a relevant error.

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
```

For fixtures, we would generate code in the exactly same manner as for simple arguments.

```cairo
// ...
#[test]
fn my_test_1() {
    let a = 1;
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
```

This could be handled in a very similar manner as the [non-parametrized fixture case](#parametrization-with-fixtures):

```cairo
// ...
#[test]
fn my_test_generated(a: felt252) {
    let a = 1;
    let b = my_fixture(2, 3, 4);
    // ...
}
```

### Parametrizing With Complex Types

While it is technically possible to use complex types in attributes instead of defining fixtures, handling of them in
code would be quite problematic based on my limited research.

Annotations do not seem to be type-checked at all.
Arguments passed to annotations are returned as AST node in the compiler code.
Handling and generating the necessary Cairo code from them is not trivial.

Based on these conclusions, it is logical we limit our support only simple types or calls to functions, without support
for nested calls, etc.

For example, this syntax would not be allowed:

```cairo
#[parametrize]
#[case(MyStruct { a: 1, b: array![2, 3] })]
#[test]
fn my_test(a: MyStruct) {
    // ...
}
```

### Problems With Code Generation

The code generation solution has some problems that will need to be addressed before it is implemented.
See [section below](#possible-problems-with-code-generation) for details.

## User Experience

Unlike fuzz-tests, we should run and indicate the result of all parametrized cases.
If some test cases fail, others should still be executed and their results should be displayed to the user.

An example output could look similarly to this:

```shell
$ snforge test
[PASS] tests::parametrized(a = 1, b = 2, c = my_fixture(1, 2))
[PASS] tests::parametrized(a = 3, b = 5, c = my_fixture(4, 5))
[FAIL] tests::parametrized(a = 4, b = 5, c = my_fixture(3, 2))

Failure data:
    original value: [344693033283], converted to a string: [PANIC]
    
[PASS] tests::parametrized(a = 5, b = 7, c = my_fixture(7, 8))
# ...
```

## Required Changes to Test Collector

To support parametric test, we need to introduce some changes to the test collector in Scarb.

### Define the Necessary Attributes

This change is mostly straightforward: It can be done in the same manner as all other attributes.

### Code Generation

If the parameterized test case uses different fixtures, we will have to generate several versions of the test case.
Additionally, if we follow the alternative approach of handling [parameterized fixtures](#parametrized-fixtures),
we will also need to generate the necessary argument definitions.

## Possible Problems With Code Generation

For each test case, we will have to generate multiple versions of the test.

This is problematic because `snforge` would treat these generated tests as completely separate entities.

Some possible solutions to this problem would be:

- Modifying the `TestCaseRaw` so it contains some additional field that allows tests cases to reference each other: This
  way would know which tests are just different cases of the same sour code.
- Modify `TestCaseRaw` so it can contain references to multiple "implementations".
  `snforge` would then find these implementations and run them in a bundle.
