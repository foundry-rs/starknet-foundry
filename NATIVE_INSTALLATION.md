# Installing Starknet Foundry With Cairo Native Support

Cairo Native introduces additional dependencies outside of the Rust ecosystem.

## LLVM

LLVM is linked into Starknet Foundry binary, so it doesn't have to be installed separately at the cost an incrased
binary size.

## `ld`

Cairo Native crate makes direct calls to `ld`, which must in turn be installed on the system.

### Linux

The package `binutils` contains `ld`, install it with package manager relevant to your distribution or build it
from source.

### MacOS

`ld` is part of the Xcode command line tools. Install it with `xcode-select --install`.
