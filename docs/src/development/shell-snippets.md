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
command: account create
add_profile: --add-profile flag was not set. No profile added to snfoundry.toml
address: 0x0[..]
message: Account successfully created but it needs to be deployed. The estimated deployment fee is [..]
        
To see account creation details, visit:
account: https://sepolia.starkscan.co/contract/[..]
```
</details>
`````

## Available configuration options

- `ignored` - if set to `true`, the snippet will be ignored and not run.
- `package_name` - the name of the Scarb package in which the snippet should be run.
- `ignored_output` - if set to `true`, the output of executed command will be ignored.
