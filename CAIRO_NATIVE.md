# Running Cairo Native

<!-- TOC -->
* [Running Cairo Native](#running-cairo-native)
  * [Installing Starknet Foundry With Cairo Native Support](#installing-starknet-foundry-with-cairo-native-support)
    * [LLVM](#llvm)
    * [`ld`](#ld)
      * [Linux](#linux)
      * [MacOS](#macos)
  * [Running Tests](#running-tests)
<!-- TOC -->

## Installing Starknet Foundry With Cairo Native Support

Cairo Native introduces additional dependencies outside of the Rust ecosystem.

### LLVM

LLVM is linked into Starknet Foundry binary, so it doesn't have to be installed separately at the cost of an increased
binary size.

### `ld`

Cairo Native crate makes direct calls to `ld`, which must in turn be installed on the system.

#### Linux

The package `binutils` contains `ld`, install it with package manager relevant to your distribution or build it
from source.

#### MacOS

`ld` is part of the Xcode command line tools. Install it with `xcode-select --install`.

## Running Tests

To run tests run `snforge test --run-native`, without the flag, the execution will still run in the VM.

When running native features that rely on VM trace like test backtrace, profiler, coverage will not work.
Running native enforces the test to run with `sierra-gas` as tracked resource. Tracking vm resources is not possible
with the native execution. 