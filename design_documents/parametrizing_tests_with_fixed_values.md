# Parametrizing Tests With Fixed Values

## Context

Various test runners for other programming languages provide ways of parametrizing test cases with fixed values.
This way, multiple test cases can be executed for the same test source code.

## Goal

Propose a way of adding support for parametrized tests to Starknet Foundry's `snforge`.

## Proposed Solution

Introduce new attribute `test_case`.

Generate code for each of the test cases so no handling of values injection, etc. is necessary.
The compiler would then validate if the values for the generated code are actually valid.
Thanks to that, we could have failures for invalid arguments early in the execution.

### Parametrization With Arguments

```cairo
#[test_case(1, 3)]
#[test_case(3, 5)]
fn my_test(a: felt252, b: u32) {
    // ...
}
```

For test cases like this, we would generate two actual test cases:

```cairo
#[test]
fn my_test_case_1() {
  let a: felt252 = 1;
  let b: u32 = 3;
  // ...
}

#[test]
fn my_test_case_2() {
    let a: felt252 = 3;
    let b: u32 = 5;
  // ...
}
```

### Generated Cases Names

Generated test cases should be named in such a way that we can automatically detect which tests are generated cases of
the same base test.

Test in `snforge` can be filtered by name.
It is important that generated test names do not break the test filtering logic.

This can be resolved by either using names for generated tests that still work with filters as the base test would do,
or changing the filtering logic, so it can recognize generated test cases and treat them accordingly.

### Parametrizing With Complex Types

Complex types would result in the same code generation as simple types:

```cairo
#[test_case(MyStruct { a: 1, b: array![2, 3] })]
fn my_test(a: MyStruct) {
    // ...
}
```

```cairo
#[test]
fn my_test_1() {
    let a = MyStruct { a: 1, b: array![2, 3] };
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
[PASS] tests::parametrized(a = 1, b = 2)
[PASS] tests::parametrized(a = 3, b = 5)
[FAIL] tests::parametrized(a = 4, b = 5)

Failure data:
    original value: [344693033283], converted to a string: [PANIC]
    
[PASS] tests::parametrized(a = 5, b = 7)
# ...
```

## Deterministic Test Output Order

Parallel test execution implementation in forge makes deterministic test execution order non-trivial.
For good user experience, we should aim to implement parametrized tests in such a way that cases results are displayed
in deterministic order and grouped together.

Test execution itself can be performed in any order as long as we display the results in a grouped manner.
We can follow the fuzzed tests implementation for this.

### Behavior of `--exit-first` Flag

In case `--exit-first` flag is used we should try to have consistent behavior of parametrized tests.

If some other test fails and there are cases of parametrized tests that did not finish execution,
we should not display any results of parametrized test.

In case a parametrized test case fails, we should display it.
Other cases may not be displayed in this case.

## Required Changes to Test Collector

To support parametric test, we need to introduce some changes to the test collector in Scarb as outlined below.

### Define the Necessary Attributes

This change is mostly straightforward: It can be done in the same manner as all other attributes.

### Code Generation

For this approach, my recommendation is to use plugins for code generation.
We could create a separate plugin for just generating test cases from our attribute and make sure it is executed before
the test collection.

Cairo repository contains multiple examples of plugins that generate Cairo code which we could follow:

- https://github.com/starkware-libs/cairo/blob/main/crates/cairo-lang-plugins/src/plugins/generate_trait.rs
- https://github.com/starkware-libs/cairo/blob/main/crates/cairo-lang-plugins/src/plugins/panicable.rs
- https://github.com/starkware-libs/cairo/blob/main/crates/cairo-lang-semantic/src/inline_macros/array.rs

Details of this are for the implementer to investigate, but my recommendation would be to not attempt any approaches
where we try to modify the test code in the compiler itself but generate the necessary test cases.

### Possible Problems With Code Generation

For each test case, we will have to generate multiple versions of the test.
This is problematic because `snforge` would treat these generated tests as completely separate entities.

Test collector should be modified, so that cases of the same test as a single bundle.
This way we can handle execution of them more easily.
