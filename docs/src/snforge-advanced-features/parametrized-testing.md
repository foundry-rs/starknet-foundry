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
{{#include ../../listings/parametrized_testing/tests/example_basic.cairo}}
```

Now run:

```shell
$ snforge test test_basic_sum
```

<details>
<summary>Output:</summary>

```shell
Collected 2 test(s) from parametrized_testing package
Running 0 test(s) from src/
Running 2 test(s) from tests/
[PASS] parametrized_testing_integrationtest::basic_example::test_basic_sum_1_2_3 ([..])
[PASS] parametrized_testing_integrationtest::basic_example::test_basic_sum_3_4_7([..])
Tests: 2 passed, 0 failed, 0 ignored, [..] filtered out
```
</details>
<br>

## Naming Test Cases

Each parameterized test gets its own generated name. There are two ways to control it:

 - **Unnamed test case** - the name is generated based on the function   name and the arguments provided.

    ```rust
    #[test_case(1, 2)]
    ``` 
    This will generate a test named `test_basic_sum_1_2`.

 - **Named test case** - you can provide a custom name for the test case using the `name` parameter.

    ```rust
    #[test_case(name: "one_plus_two", 1, 2)]
    ```
    This will generate a test named `test_basic_sum_one_plus_two`.

## More Complex Example

Now let's look at a more complex example which uses structs as parameters.

```rust
{{#include ../../listings/parametrized_testing/tests/example_complex.cairo}}
```

Now run:

```shell
$ snforge test test_is_adult
```

<details>
<summary>Output:</summary>

```shell
Collected 3 test(s) from parametrized_testing package
Running 3 test(s) from tests/
[PASS] parametrized_testing_integrationtest::example_complex::test_is_adult_user_name_alice_age_20_true ([..])
[PASS] parametrized_testing_integrationtest::example_complex::test_is_adult_user_name_josh_age_18_true ([..])
[PASS] parametrized_testing_integrationtest::example_complex::test_is_adult_user_name_bob_age_14_false ([..])
Running 0 test(s) from src/
Tests: 3 passed, 0 failed, 0 ignored, [..] filtered out
```
</details>
<br>

## Combining With Other Attributes

`#[test_case]` can be freely combined with other attributes.

Below is an example of a parameterized test that also uses the fuzzer.

```rust
{{#include ../../listings/parametrized_testing/tests/example_with_fuzzer.cairo}}
```

Now run:

```shell
$ snforge test test_basic_sum_with_fuzzer
```

<details>
<summary>Output:</summary>

```shell
Collected 3 test(s) from parametrized_testing package
Running 3 test(s) from tests/
[PASS] parametrized_testing_integrationtest::example_with_fuzzer::sum_with_fuzzer_1_2_3 ([..])
[PASS] parametrized_testing_integrationtest::example_with_fuzzer::sum_with_fuzzer_3_4_7 ([..])
[PASS] parametrized_testing_integrationtest::example_with_fuzzer::sum_with_fuzzer ([..])
Running 0 test(s) from src/
Tests: 3 passed, 0 failed, 0 ignored, [..] filtered out
Fuzzer seed: [..]
```
</details>
<br>