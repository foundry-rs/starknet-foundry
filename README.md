# Starknet Foundry 🔨

Blazingly fast implementation of Foundry for developing Starknet contracts designed & developed by ex [Protostar](https://github.com/software-mansion/protostar) team from [Software Mansion](https://github.com/software-mansion/protostar)

## Installation

To install Starknet-Foundry, run:

```shell
curl -L https://raw.githubusercontent.com/foundry-rs/starknet-foundry/master/install.sh | bash
```

If you want to specify a version, run the following command with the requested version:

```shell
curl -L https://raw.githubusercontent.com/foundry-rs/starknet-foundry/master/install.sh | bash -s -- -v 0.1.0
```

To check if the Starknet-Foundry is installed correctly, run `forge -v` and `cast -v`.

## Development

### Environment setup

TODO

### Testing
Test scripts require you to have asdf installed. 
Moreover, Cast's tests require devenet.
```bash
$ ./scripts/test_forge.sh
$ ./scripts/test_cast.sh
```
