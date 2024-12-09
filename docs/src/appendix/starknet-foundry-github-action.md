# Starknet Foundry Github Action

If you wish to use Starknet Foundry in your Github Actions workflow, you can use the [`setup-snfoundry`](https://github.com/marketplace/actions/setup-starknet-foundry) action. This action installs the necessary `snforge` and `sncast` binaries. At this moment, it only supports Linux and MacOS.

## Example workflow

```yml
{{#include ../../example_workflows/basic_workflow.yml}}
```

## Caching

In order to optimize the workflow, you should cache Starknet Foundry. Here is an example of how to do it:

```yaml
{{#include ../../example_workflows/workflow_with_cache.yml}}
```
