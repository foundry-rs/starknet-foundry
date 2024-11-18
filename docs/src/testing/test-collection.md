# How Tests Are Collected

Snforge executes tests, but it does not compile them directly.
Instead, it compiles tests by internally running `scarb build --test` command.

The `snforge_scarb_plugin` dependency, which is included with `snforge_std` dependency makes all functions
marked with `#[test]` executable and indicates to Scarb they should be compiled.
Without the plugin, no snforge tests can be compiled, that's why `snforge_std` dependency is always required in all
snforge projects.

Thanks to that, Scarb collects all functions marked with `#[test]`
from [valid locations](https://docs.swmansion.com/scarb/docs/extensions/testing.html#tests-organization) and compiles
them into tests that are executed by snforge.

## `[[test]]` Target

Under the hood, Scarb utilizes the `[[test]]` target mechanism to compile the tests. More information about the
`[[test]]` target is available in
the [Scarb documentation](https://docs.swmansion.com/scarb/docs/reference/targets.html#test-targets).

By default, `[[test]]]` target is implicitly configured and user does not have to define it.
See [Scarb documentation](https://docs.swmansion.com/scarb/docs/reference/targets.html#auto-detection-of-test-targets)
for more details about the mechanism.

## Tests Organization

Test can be placed in both `src` and `test` directories. When adding tests to files in `src` you must wrap them in tests
module.

You can read more about tests organization
in [Scarb documentation](https://docs.swmansion.com/scarb/docs/extensions/testing.html#tests-organization).

### Unit Tests

Test placed in `src` directory are often called unit tests.
For these test to function in snforge, they must be wrapped in a module marked with `test` attribute.

```rust
// src/example.rs
// ...

// This test is not in module marked with `#[cfg(test)]` so it won't work
#[test]
fn my_invalid_test() {
    // ...
}

#[cfg(test)]
mod tests {
    // This test is in module marked with `#[cfg(test)]` so it will work
    #[test]
    fn my_test() {
        // ..
    }
}
```

### Integration Tests

Integration tests are placed in `tests` directory.
This directory is a special directory in Scarb.
Tests do not have to be wrapped in `#[cfg(test)]` and each file is treated as a separate module.

```rust
// tests/example.rs
// ...

// This test is in `tests` directory
// so it works without being in module with `#[cfg(test)]` 
#[test]
fn my_test_1() {
    // ..
}
```

#### Modules and `lib.cairo`

As written above, each file in `tests` directory is treated as a separate module

```shell
$ tree
tests/
├── module1.cairo <-- is collected
├── module2.cairo <-- is collected
└── module3.cairo <-- is collected
```

Scarb will collect each of the file and compile it as a
separate [test target](https://docs.swmansion.com/scarb/docs/reference/targets.html#test-targets).
Each of this target will be run separately by `snforge`.

However, it is also possible to define `lib.cairo` file in `tests`.
This stops files in `tests` from being treated as separate modules.
Instead, Scarb will only create a single test target for that `lib.cairo` file.
Only tests that are reachable from this file will be collected and compiled.

```shell
$ tree
tests/
├── lib.cairo
├── module1.cairo  <-- is collected
├── module2.cairo  <-- is collected
└── module3.cairo  <-- is not collected
```

```rust
// tests/lib.cairo

mod module1;
mod module2;
```
