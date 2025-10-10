# Parametrized Testing

Sometimes, you want to run the same test logic with different inputs.
Instead of duplicating code into separate test functions, you can use parameterized tests.

Parameterized tests allow you to define multiple test cases for a single function by attaching the
`#[test_case]` attribute.

Each test case provides its own set of arguments, and `snforge` will automatically generate separate test instances for them.

## Basic Example

To turn a regular test into a parameterized one, add the `#[test_case(...)]` attribute above it.
You can provide any valid Cairo expressions as arguments.

Below is a simple example which checks addition of two numbers.

```rust
{{#include ../../listings/parametrized_testing_basic/tests/example.cairo}}
```

Now run:

<!-- { "package_name": "parametrized_testing_basic", "scarb_version": ">=2.12.0" } -->
```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 2 test(s) from parametrized_testing_basic package
Running 0 test(s) from src/
Running 2 test(s) from tests/
[PASS] parametrized_testing_basic_integrationtest::example::test_sum_1_2_3 ([..])
[PASS] parametrized_testing_basic_integrationtest::example::test_sum_3_4_7 ([..])
Tests: 2 passed, 0 failed, 0 ignored, [..] filtered out
```
</details>
<br>

## Naming Test Cases

Each parameterized test gets its own generated name. There are two ways to control it:

 - **Unnamed test case** - the name is generated based on the function name and the arguments provided.

    ```rust
    #[test_case(1, 2, 3)]
    fn test_sum(x: felt252, y: felt252, expected: felt252) {
        assert_eq!(sum(x, y), expected);
    }
    ``` 
    This will generate a test named `test_sum_1_2_3`.

 - **Named test case** - you can provide a custom name for the test case using the `name` parameter.

    ```rust
    #[test_case(name: "one_plus_two", 1, 2, 3)]
    fn test_sum(x: felt252, y: felt252, expected: felt252) {
        assert_eq!(sum(x, y), expected);
    }
    ```
    This will generate a test named `test_sum_one_plus_two`.

> 📝 **Note**
> For unnamed test cases, it's possible that two different input values of the same type can generate the same test case name.
> In such cases we emit a diagnostic error.
> To resolve it, simply provide an explicit `name` for the case.

## Advanced Example

Now let's look at an addvanced example which uses structs as parameters.

```rust
{{#include ../../listings/parametrized_testing_advanced/tests/example.cairo}}
```

Now run:

<!-- { "package_name": "parametrized_testing_advanced", "scarb_version": ">=2.12.0" } -->
```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 3 test(s) from parametrized_testing_advanced package
Running 3 test(s) from tests/
[PASS] parametrized_testing_advanced_integrationtest::example::test_is_adult_user_name_alice_age_20_true ([..])
[PASS] parametrized_testing_advanced_integrationtest::example::test_is_adult_user_name_josh_age_18_true ([..])
[PASS] parametrized_testing_advanced_integrationtest::example::test_is_adult_user_name_bob_age_14_false ([..])
Running 0 test(s) from src/
Tests: 3 passed, 0 failed, 0 ignored, [..] filtered out
```
</details>
<br>

## Combining With Fuzzer Attribute

`#[test_case]` can be freely combined with the `#[fuzzer]` attribute.

Below is an example in which we will fuzz the test but also run the specific defined cases.

```rust
{{#include ../../listings/parametrized_testing_fuzzer/tests/example.cairo}}
```

Now run:

<!-- { "package_name": "parametrized_testing_fuzzer", "scarb_version": ">=2.12.0" } -->
```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 3 test(s) from parametrized_testing_fuzzer package
Running 3 test(s) from tests/
[PASS] parametrized_testing_fuzzer_integrationtest::example::test_sum_1_2 ([..])
[PASS] parametrized_testing_fuzzer_integrationtest::example::test_sum_3_4 ([..])
[PASS] parametrized_testing_fuzzer_integrationtest::example::test_sum ([..])
Running 0 test(s) from src/
Tests: 3 passed, 0 failed, 0 ignored, 0 filtered out
Fuzzer seed: [..]
```
</details>
<br>