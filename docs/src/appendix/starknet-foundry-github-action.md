# Starknet Foundry Github Action

If you wish to use Starknet Foundry in your Github Actions workflow, you can use the [setup-snfoundry](https://github.com/marketplace/actions/setup-starknet-foundry) action. This action installs the necessary `snforge` and `sncast` binaries.

> 📝 **Note**
> At this moment, only Linux and MacOS are supported.

## Example workflow

Make sure you pass the valid path to `Scarb.lock` to [setup-scarb](https://github.com/marketplace/actions/setup-scarb) action. This way, all dependencies including snforge_scarb_plugin will be cached between runs.

```yml
{{#include ../../example_workflows/basic_workflow.yml}}
```

## Workflow With Partitioned Tests

If you have a large number of tests, you can speed up your CI by partitioning tests and running them in parallel jobs. Here's an example workflow that demonstrates how to achieve this:

```yml
{{#include ../../example_workflows/partitioned_workflow.yml}}
```

Read more about [tests partitioning here](../snforge-advanced-features/tests-partitioning.md).
