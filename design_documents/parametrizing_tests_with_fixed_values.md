# Parametrizing Tests With Fixed Values

## Context

Various test runners for other programming languages provide ways of parametrizing test cases with fixed values.
This way, multiple test cases can be executed for the same test source code.

## Goal

Propose a way of adding support for parametrized tests to Starknet Foundry's `snforge`.

## Proposed Solution

Introduce new attribute `test_case`.

The attribute should accept arguments:

- Optional, named argument `name`.
- Unnamed, non-optional arguments, each corresponding to a test argument.

Generate code for each of the test cases so no handling of values injection, etc. is necessary.
The compiler would then validate if the values for the generated code are actually valid.
Thanks to that, we could have failures for invalid arguments early in the execution.

### Parametrization With Arguments

In the `test_case` attribute, user provides a list of unnamed arguments, each corresponding to test function argument.
Arguments are handled in order: The first argument of the attribute corresponds to the first argument of a test
function.

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

For consistent UX, we should not allow filtering specific test cases.
It should only be possible to filter the whole parametrized test.

This can be resolved by either using names for generated tests that still work with filters as the base test would do,
or changing the filtering logic, so it can recognize generated test cases and treat them accordingly.

### Named Test Cases

If `name` attribute is provided, instead of generating case name, it should be used instead.
For `name` values to be suitable for generating case names, they are limited to lowercase letters, digits and
underscore `_` character.
Provided names must be unique and must not be the same as automatically generated names of any of the test cases.

For example:

```cairo
#[test_case(1, 3, name: "my_name")]
#[test_case(3, 5, name: "a_test")]
fn my_test(a: felt252, b: u32) {
    // ...
}
```

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

If a test case has a custom name, it should be displayed to the user.

An example output could look similarly to this:

```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
[PASS] tests::parametrized(a = 1, b = 2)              # unnamed test case
[PASS] tests::parametrized[a_test](a = 3, b = 5)      # named test case
[FAIL] tests::parametrized[my_case](a = 4, b = 5)     # named test case

Failure data:
    0x50414e4943 ('PANIC')
    
[PASS] tests::parametrized(a = 5, b = 7)
# ...
```
</details>
<br>

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

### Behavior of `#[ignore]` Attribute

Adding `#[ignore]` attribute to parametrized test should ignore all of it's test cases - test should not run at all.

### Behavior of `--rerun-failed` Flag

In case `--rerun-failed` flag is used, only failed cases of a parametrized test should be rerun.

Rerun cases should still be displayed together: The behavior should be exactly the same as if just running test cases.

### Behavior of `#[fuzzer]` Attribute

It must not be possible to add `#[fuzzer]` attribute to parametrized test.
It must not be possible to use fuzzing in parametrized test.

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

Details of this are for the implementer to investigate.
My recommendation is to follow the implementations of other Cairo plugins and generate Cairo code for test cases.

### Possible Problems With Code Generation

For each test case, we will have to generate multiple versions of the test.
This is problematic because `snforge` would treat these generated tests as completely separate entities.

Test collector should be modified, so that cases of the same test as a single bundle.
This way we can handle execution of them more easily.

## Implementation Plan

1. Prototype creating a plugin for test code generation.
   Initially, it can support just simple arguments and unnamed test cases.
2. If/Once code generation approach is proven viable, add support for complex types arguments and introduce necessary
   test collection logic according to the document.
3. Named test cases can be added after MVP is created.
