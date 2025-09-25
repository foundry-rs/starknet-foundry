# Shell Snippets in Documentation

`snforge` and `sncast` snippets and their outputs present in the docs are automatically run and tested. Some of them need to be configured with specific values to work correctly.

## Snippet configuration

To configure a snippet, you need to add a comment block right before it. The comment block should contain the configuration in JSON format. Example:

`````markdown
<!-- { "package_name": "hello_starknet", "ignored_output": true } -->
```shell
$ sncast \
    account create \
    --network sepolia \
    --name my_first_account
```

<details>
<summary>Output:</summary>

```shell
Success: Account created

Address: 0x[..]

Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee
   
To see account creation details, visit:
account: https://sepolia.starkscan.co/contract/[..]
```
</details>
`````

## Available configuration options

- `ignored` - if set to `true`, the snippet will be ignored and not run.
- `package_name` - the name of the Scarb package in which the snippet should be run.
- `ignored_output` - if set to `true`, the output of executed command will be ignored.
- `replace_network` - if set to `true`, the snippet will replace the `--network` argument with devnet used in tests.
- `scarb_version` - specifies the Scarb version required to run the snippet. If the current Scarb version does not match, the snippet will be ignored. The version should be in the format compatible with [semver](https://semver.org/).
