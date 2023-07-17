# Contribution guideline

Starknet Foundry is actively developed and open for contributions! Want to get started? Grab any issue labeled with `good-first-issue`!
Need some guidance? Reach out to other developers on Telegram or open a GitHub discussion!

## Developer environment

### Environment setup

1. Install the latest [Rust](https://www.rust-lang.org/tools/install) version.
If you already have Rust installed make sure to upgrade it by running
```shell
$ rustup update
```
2. Clone this repository
3. Verify your setup by running [tests](#testing)
4. Build Starknet Foundry
```shell
$ cd ./starknet-foundry && cargo build --bins --release
```

### Testing
Test scripts require you to have asdf installed. 
Cast's tests require devenet as well.
Moreover, `./scripts/prepare-for-tests.sh` should be run once after setting up the development environment.

```bash
$ ./scripts/test_forge.sh
$ ./scripts/test_cast.sh
```
