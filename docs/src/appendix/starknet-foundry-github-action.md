# Starknet Foundry Github Action

If you wish to use Starknet Foundry in your Github Actions workflow, you can use the [setup-snfoundry](https://github.com/marketplace/actions/setup-starknet-foundry) action. This action installs the necessary `snforge` and `sncast` binaries.

> 📝 **Note**
> At this moment, only Linux and MacOS are supported.

## Example workflow

Make sure you pass the valid path to `Scarb.lock` to [setup-scarb](https://github.com/marketplace/actions/setup-scarb) action. This way, all dependencies including snforge_scarb_plugin will be cached between runs.

```yml
{{#include ../../../example_workflows/basic_workflow.yml}}
```
