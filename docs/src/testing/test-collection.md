## Test Collection

Forge considers all functions in your project marked with `#[test]` attribute as tests.
By default, test functions run without any arguments.
However, adding any arguments to function signature will enable [fuzz testing](./advanced/fuzz-testing.md) for this
test case.

Starknet Forge will collect tests only from these places:

- any files reachable from the package root (declared as `mod` in `lib.cairo` or its children)
- files inside the [`tests`](#the-tests-directory) directory

## The *tests* Directory

Forge collects tests from `tests` directory.
Depending on the presence of `tests/lib.cairo` file, the behavior of the test collector will be different.

### With *tests/lib.cairo*

If there is a `lib.cairo` file in `tests` folder,
then it is treated as an entrypoint to the `tests` package from which tests are collected.

For example, for a package structured this way:

```shell
$ tree .
.
├── Scarb.toml
├── tests/
│   ├── lib.cairo
│   ├── common/
│   │   └── utils.cairo
│   ├── common.cairo
│   ├── test_contract.cairo
│   └── not_included.cairo
└── src/
    └── lib.cairo
```

with `tests/lib.cairo` content:

```rust
mod common;
mod test_contract;
```

and `tests/common.cairo` content:

```rust
mod utils;
```

tests from `tests/lib.cairo`, `tests/test_contract.cairo`, `tests/common.cairo`
and `tests/common/utils.cairo` will be collected.

### Without *tests/lib.cairo*

When there is no `lib.cairo` present in `tests` folder, 
all test files **directly** in `tests` directory (i.e., not in its subdirectories)
are treated as modules and added to a single virtual `lib.cairo`. 
Then this virtual `lib.cairo` is treated as an entrypoint to the `tests` package from which tests are collected.

For example, for a package structured this way:

```shell
$ tree .
.
├── Scarb.toml
├── tests/
│   ├── common/
│   │   └── utils.cairo
│   ├── common.cairo
│   ├── test_contract.cairo
│   └── not_included/
│       └── ignored.cairo
└── src/
    └── lib.cairo
```

and `tests/common.cairo` content:

```rust
mod utils;
```

tests from `tests/test_contract.cairo`, `tests/common.cairo` and `tests/common/utils.cairo` will be collected.

### Sharing Code Between Tests

Sometimes you may want a share some code between tests to organize them. 
The package structure of tests makes it easy! 
In both of the above examples, you can
make the functions from `tests/common/utils.cairo` available in `tests/test_contract.cairo` by using:
- an absolute import: `use tests::common::utils;`
- a relative import: `use super::common::utils;`
