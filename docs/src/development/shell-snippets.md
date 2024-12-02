# Shell Snippets in Documentation

`snforge` and `sncast` snippets and their outputs present in the docs are automatically run and tested. Some of them need to be configured with specific values to work correctly.

## Snippet configuration

To configure a snippet, you need to add a comment block right before it. The comment block should contain the configuration in JSON format. Example:

```markdown
<!-- { "package_name": "hello_starknet", "ignore_output": true } -->
```

## Available configuration options

- `ignored` - if set to `true`, the snippet will be ignored and not run.
- `package_name` - the name of the Scarb package in which the snippet should be run.
- `contract_name` - the name of the contract which snippet should use. If present, class hash of predefined contract will be used instead of the one from snippet.
- `ignored_output` - if set to `true`, the output of executed command will be ignored.
