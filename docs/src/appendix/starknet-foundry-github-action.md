# Starknet Foundry Github Action

If you wish to use Starknet Foundry in your Github Actions workflow, you can use the [setup-snfoundry](https://github.com/marketplace/actions/setup-starknet-foundry) action. This action installs the necessary `snforge` and `sncast` binaries.

> 📝 **Note**
> At this moment, it only supports Linux and MacOS.

## Example workflow

```yml
{{#include ../../example_workflows/basic_workflow.yml}}
```

## Caching

In order to optimize the workflow, make sure you pass the valid path to `Scarb.lock` to [setup-scarb](https://github.com/marketplace/actions/setup-scarb) action. This way, all dependencies including snforge_scarb_plugin will be cached between runs.

```yaml
{{#include ../../example_workflows/workflow_with_cache.yml}}
```
