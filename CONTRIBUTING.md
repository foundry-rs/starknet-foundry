# Contribution Guideline

Starknet Foundry is actively developed and open for contributions!
Want to get started?
Grab any issue labeled with `good-first-issue`!
Need some guidance?

[//]: # (TODO add telegram and gh discussions links)

Reach out to other developers on [Telegram]() or open a [GitHub discussion]()!

### Environment setup

See [development guide](https://foundry-rs.github.io/starknet-foundry/development/environment-setup.html) in Starknet
Foundry book for environment setup.

### Running Tests and Checks

Test scripts require you to have asdf installed.
Check out [asdf docs](https://asdf-vm.com/guide/getting-started.html) for more details.

⚠️ Make sure you run `./scripts/prepare-for-tests.sh` after setting up the development environment, otherwise tests will
fail.

Before creating a contribution, make sure your code passes the following checks

```shell
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- --no-deps -W clippy::pedantic -A clippy::missing_errors_doc -A clippy::missing_panics_doc -A clippy::default_trait_acces
```

Otherwise, it won't be possible to merge your contribution.

## Contributing

Before you open a pull request, it is always a good idea to search the issues and verify if the feature you would like
to add hasn't been already discussed.
We also appreciate creating a feature request before making a contribution, so it can be discussed before you get to
work.

### Writing Tests

Please make sure the feature you are implementing is thoroughly tested with automatic tests.
You can check existing tests in the repository to see the recommended approach to testing.

### Breaking Changes

If the change you are introducing is changing or breaking the behavior of any already existing features, make sure to
include that information in the pull request description.

