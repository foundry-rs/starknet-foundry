# Starknet Foundry 🔨

Blazing fast toolkit for developing Starknet contracts designed & developed by
ex [Protostar](https://github.com/software-mansion/protostar) team from [Software Mansion](https://swmansion.com) based
on native [Cairo](https://github.com/starkware-libs/cairo) test runner
and [Blockifier](https://github.com/starkware-libs/blockifier), written in Rust 🦀.

Need help getting started with Starknet Foundry? Read the
📖 [Starknet Foundry Book](https://foundry-rs.github.io/starknet-foundry/)!


![Example run](./docs/images/demo-gif/demo.gif)


Starknet Foundry, like its [Ethereum counterpart](https://github.com/foundry-rs/foundry), consists of different modules

- [Forge](https://github.com/foundry-rs/starknet-foundry/tree/master/starknet-foundry/crates/forge): Starknet testing
  framework (like Truffle, Hardhat and DappTools but for Starknet).
- [Cast](https://github.com/foundry-rs/starknet-foundry/tree/master/starknet-foundry/crates/cast): All-in-one tool for
  interacting with Starknet smart contracts, sending transactions and getting chain data.

## Features

- Fast testing framework `Forge` written in Rust
- High-quality dependency management using [scarb](https://github.com/software-mansion/scarb)
- Intuitive interactions and deployment of Starknet contracts

## Coming Soon 👀

Starknet Foundry is under active development! Expect a lot of new features to appear soon! 🔥

- [ ] Cheatcodes
- [ ] Performance improvements
- [ ] State forking
- [ ] Advanced debugging utilities
- [ ] Creating and deploying new accounts
- [ ] Deployment scripts written in Cairo

## Performance

![Starknet test framework speed comparison](./benchmarks/plot.png)

Forge achieves performance comparable to the Cairo Test Runner with greatly improved user experience. All that is possible on just a single thread and multithreading is well on it's way!

To learn more about our benchmark methodology check [here](./benchmarks/).

## Getting Help

You haven't found your answer to your question in
the [Starknet Foundry Book](https://foundry-rs.github.io/starknet-foundry/)?

- Join the [Telegram](https://t.me/+d8ULaPxeRqlhMDNk) group to get help
- Open a [GitHub discussion](https://github.com/foundry-rs/starknet-foundry/discussions) with your question
- Join the [Starknet Discord](https://discord.com/invite/qypnmzkhbc)

Found a bug? Open an [issue](https://github.com/foundry-rs/starknet-foundry/issues).

## Contributions

Starknet Foundry is under active development, and we appreciate any help from the community! Want to contribute? Read
the [contribution guidelines](./CONTRIBUTING.md).

Check out [development guide](https://foundry-rs.github.io/starknet-foundry/development/environment-setup.html) for
local environment setup guide.