# Starknet Foundry Book

## Installation

Install mdBook

```shell
$ cargo install mdbook
```

Install necessary mdBook extensions

```shell
$ cargo install mdbook-linkcheck2
$ ln -sf ~/.cargo/bin/mdbook-linkcheck2 ~/.cargo/bin/mdbook-linkcheck
```

```shell
$ cargo install mdbook-variables
```

## Building

```shell
$ mdbook build
```

## Open preview and reload on every change

```shell
$ mdbook serve
```