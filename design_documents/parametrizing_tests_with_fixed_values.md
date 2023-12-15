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

Alternatively, we could generate code where arguments of `my_fixture` are injected:

```cairo
// ...
#[test]
fn my_test_generated(a: felt252, my_fixture_a: felt252, my_fixture_b: felt252, my_fixture_c: felt252) {
    let b = my_fixture(my_fixture_a, my_fixture_b, my_fixture_c);
    // ...
}
```

This has the benefit of only needing to generate one version of the test. Necessary arguments just need to be injected.

The code generation solution has some problems that will need to be addressed before it is implemented.
See [section below](#possible-problems-with-code-generation) for details.

## Required Changes to Test Collector

To support parametric test, we need to introduce some changes to the test collector in Scarb.

### Define the Necessary Attributes

This change is mostly straightforward: It can be done in the same manner as all other attributes.

### Code Generation

If the parameterized test case uses different fixtures, we will have to generate several versions of the test case.
Additionally, if we follow the alternative approach of handling [parametrized fixtures](#parametrized-fixtures),
we will also need to generate the necessary argument definitions.

### Argument Injection

The structure representing a test case (currently named `TestCaseRaw`), should gain additional field `cases` containing
arguments for each of the parametrized cases.

```cairo
struct TestCaseRaw {
    // ...
    cases: Vec<Vec<String>>
}
```

Where each `Vec<String>` represents arguments for a single case.
Arguments should be only defined using primitive types.

These arguments could be then handled in `snforge` and injected in the same manner as fuzzer arguments are.

## Possible Problems With Code Generation

In case the parametrized test case uses different fixtures in different cases, we will have to generate multiple
versions of the test case.

This is problematic because `snforge` would treat these generated tests as completely separate entities.

Some possible solutions for this problem would be:

- Modifying the `TestCaseRaw` so it contains some additional field that allow tests cases to reference each other: This
  way would know which tests are just different cases of the same sour code.
- Modify `TestCaseRaw` so it can contain references to multiple "implementations".
  `snforge` would then find these implementations and run them in a bundle.
