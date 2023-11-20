<img src="./docs/src/images/logo.png" alt="logo" width="120" align="right" />

## Starknet Foundry

[![Telegram Chat][tg-badge]][tg-url] [![Telegram Support][tg-support-badge]][tg-support-url]

[tg-badge]: https://img.shields.io/endpoint?color=neon&logo=telegram&label=chat&style=flat-square&url=https%3A%2F%2Ftg.sumanjay.workers.dev%2Fstarknet_foundry
[tg-url]: https://t.me/starknet_foundry
[tg-support-badge]: https://img.shields.io/endpoint?color=neon&logo=telegram&label=support&style=flat-square&url=https%3A%2F%2Ftg.sumanjay.workers.dev%2Fstarknet_foundry_support
[tg-support-url]: https://t.me/starknet_foundry_support


Blazingly fast toolkit for developing Starknet contracts designed & developed by
ex [Protostar](https://github.com/software-mansion/protostar) team from [Software Mansion](https://swmansion.com) based
on native [Cairo](https://github.com/starkware-libs/cairo) test runner
and [Blockifier](https://github.com/starkware-libs/blockifier), written in Rust 🦀.

Need help getting started with Starknet Foundry? Read the
📖 [Starknet Foundry Book](https://foundry-rs.github.io/starknet-foundry/)!

![Example run](.github/images/demo-gif/demo.gif)

Starknet Foundry, like its [Ethereum counterpart](https://github.com/foundry-rs/foundry), consists of different modules

- [Forge](https://github.com/foundry-rs/starknet-foundry/tree/master/crates/forge): Starknet testing
  framework (like Truffle, Hardhat and DappTools but for Starknet).
- [Cast](https://github.com/foundry-rs/starknet-foundry/tree/master/crates/cast): All-in-one tool for
  interacting with Starknet smart contracts, sending transactions and getting chain data.

## Installation

To install Starknet Foundry, run:

```shell
curl -L https://raw.githubusercontent.com/foundry-rs/starknet-foundry/master/scripts/install.sh | sh
```

You can also specify a version you wish to install:

```shell
curl -L https://raw.githubusercontent.com/foundry-rs/starknet-foundry/master/scripts/install.sh | sh -s -- -v 0.3.0
```

To verify that the Starknet Foundry is installed correctly, run `snforge --version` and `sncast --version`.


## Features

- Fast testing framework `Forge` written in Rust
- High-quality dependency management using [scarb](https://github.com/software-mansion/scarb)
- Intuitive interactions and deployment of Starknet contracts through `Cast`

## Roadmap

Starknet Foundry is under active development! Expect a lot of new features to appear soon! 🔥

- [x] Running tests written in Cairo
- [x] Contract interactions testing
- [x] Interacting with Starknet from command line
- [x] Multicall support
- [x] Cheatcodes
- [x] Starknet state forking
- [x] Fuzz testing
- [x] Parallel tests execution
- [x] Performance improvements
- [ ] Deployment scripts written in Cairo
- [ ] Advanced debugging utilities
- [ ] L1 ↔ L2 messaging and cross-chain testing
- [ ] Transactions profiling
- [ ] Test coverage reports

## Performance

Forge achieves performance comparable to the Cairo Test Runner with improved user experience. All that is possible on just a single thread and multithreading is well on its way!

![Starknet test framework speed comparison](./benchmarks/plot.png)

To learn more about our benchmark methodology check [here](./benchmarks/).

## Getting Help

You haven't found your answer to your question in
the [Starknet Foundry Book](https://foundry-rs.github.io/starknet-foundry/)?

- Join the [Telegram](https://t.me/starknet_foundry_support) group to get help
- Open a [GitHub discussion](https://github.com/foundry-rs/starknet-foundry/discussions) with your question
- Join the [Starknet Discord](https://discord.com/invite/qypnmzkhbc)

Found a bug? Open an [issue](https://github.com/foundry-rs/starknet-foundry/issues).

## Contributions

Starknet Foundry is under active development, and we appreciate any help from the community! Want to contribute? Read
the [contribution guidelines](./CONTRIBUTING.md).

Check out [development guide](https://foundry-rs.github.io/starknet-foundry/development/environment-setup.html) for
local environment setup guide.
