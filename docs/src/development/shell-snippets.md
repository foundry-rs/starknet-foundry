# Shell Snippets in Documentation

`snforge` and `sncast` snippets and their outputs present in the docs are automatically run and tested. Some of them need to be configured with specific values to work correctly.

## Snippet configuration

To configure a snippet, you need to add a comment block right before it. The comment block should contain the configuration in JSON format. Example:

`````markdown
<!-- { "ignored": "ignore_output": true } -->
```shell
$ sncast \
    account create \
    --url http://127.0.0.1:5055 \
    --name my_first_account
```

<details>
<summary>Output:</summary>

```shell
command: account create
add_profile: --add-profile flag was not set. No profile added to snfoundry.toml
address: [..]
max_fee: [..]
message: Account successfully created. Prefund generated address with at least <max_fee> STRK tokens or an equivalent amount of ETH tokens. It is good to send more in the case of higher demand.

To see account creation details, visit:
account: https://sepolia.starkscan.co/contract/[..]
```
</details>
`````

## Available configuration options

- `ignored` - if set to `true`, the snippet will be ignored and not run.
- `package_name` - the name of the Scarb package in which the snippet should be run.
- `contract_name` - the name of the contract which snippet should use. If present, class hash of predefined contract will be used instead of the one from snippet.
- `ignored_output` - if set to `true`, the output of executed command will be ignored.
