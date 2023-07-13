# First Steps With Starknet Foundry

In this section we provide and overview of Starknet Foundry `forge` command line tool. We demonstrate how to create a
new project, compile, and test it.

To start a new project with Starknet Foundry, clone the template repository

```shell
$ git clone https://github.com/foundry-rs/starknet_forge_template.git
```

Let's check out the project structure

```shell
$ cd starknet_forge_template
$ tree . -L 1
.
├── README.md
├── Scarb.toml
├── src
└── tests

3 directories
```

We can build a project with `scarb build`

```shell
$ scarb build
   Compiling starknet_forge_template v0.1.0 (/starknet_forge_template/Scarb.toml)
    Finished release target(s) in 0 seconds
```

And run tests with `forge`

```shell
$ forge
Collected 4 test(s) and 2 test file(s)
Running 1 test(s) from src/lib.cairo
[PASS] src::test_fib
Running 3 test(s) from tests/lib_test.cairo
[PASS] lib_test::lib_test::test_fib
[PASS] lib_test::lib_test::test_simple
original value: [6381921], converted to a string: [aaa]
[PASS] lib_test::lib_test::test_print
Tests: 4 passed, 0 failed, 0 skipped
```

> 💡 **Tip**
> Running `scarb build` before `forge` is not required. Forge will automatically rebuild your project and tests.