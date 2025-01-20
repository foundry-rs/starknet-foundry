# Contribution Guideline

Starknet Foundry is under active development and is open for contributions!

## Opening an issue 

If you think something doesn't work or something is missing please open an issue! This way we can address this problem
and make Starknet Foundry better!
Before opening an issue, it is always a good idea to search existing 
[issues](https://github.com/foundry-rs/starknet-foundry/issues) and verify if a similar one doesn't already exist. 


## Contributing

### Environment setup

See [development guide](https://foundry-rs.github.io/starknet-foundry/development/environment-setup.html) in Starknet
Foundry book for environment setup.

### Selecting an issue
If you are a first time contributor pick up any issue labeled as `good-first-issue`. Write a comment that you would like to 
work on it and we will assign it to you. Need some guidance? Reach out to other developers on [Telegram](https://t.me/+d8ULaPxeRqlhMDNk).

If you are a more experienced Starknet Foundry contributor you can pick any issue labeled as `help-wanted`. Make sure to discuss the details with the core team beforehand.

### Writing Tests

Please make sure the feature you are implementing is thoroughly tested with automatic tests.
You can check existing tests in the repository to see the recommended approach to testing.

### Pull Request Size

Try to make your pull request self-contained, only introducing the necessary changes.
If your feature is complicated,
consider splitting the changes into meaningful parts and introducing them as separate pull requests.

Creating very large pull requests may significantly increase review time or even prevent them from being merged.

### Contributions Related to Spelling and Grammar

At this time, we will not be accepting contributions that only fix spelling or grammar errors in documentation, code or
elsewhere.

### `sncast` Guidelines

#### Command Outputs

Please follow these rules when creating outputs for `sncast`:

- Use an imperative tone
- Keep your message concise and to the point
- When displaying config, use `key: value` format
- If the executed command has a natural successor-command, display it as hint in the output. For example, the output of `declare` command should include a hint to use `deploy` command next.
<!-- TODO(#2859): Add bullet point about colors used for text when displaying fees, addresses and hashes -->
