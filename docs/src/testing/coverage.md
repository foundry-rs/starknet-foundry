# Coverage

Coverage reporting allows developers to gain comprehensive insights into how their code is executed.
With `cairo-coverage`, you can generate a coverage report that can later be analyzed for detailed coverage statistics.

## Prerequisites

`cairo-coverage` relies on debug information provided by Scarb. To generate the necessary debug information, you need to have:

1. [Scarb](https://github.com/software-mansion/scarb) version `2.8.0` or higher
2. `Scarb.toml` file with the following Cairo compiler configuration:
```toml
[profile.dev.cairo]
unstable-add-statements-code-locations-debug-info = true
unstable-add-statements-functions-debug-info = true
inlining-strategy = "avoid"
```

> 📝 **Note**
>
> That `unstable-add-statements-code-locations-debug-info = true` and
`unstable-add-statements-functions-debug-info = true` will slow down the compilation and cause it to use more system
> memory. It will also make the compilation artifacts larger. So it is only recommended to add these flags when you need
> their functionality.

For more information about these sections, please refer to the [Scarb documentation](https://docs.swmansion.com/scarb/docs/reference/manifest.html#cairo).

## Installation and usage

In order to run coverage report with `cairo-coverage` you need to install it first. 
Please refer to the instructions provided in the README for guidance:
<https://github.com/software-mansion/cairo-coverage#installation>

Usage details and limitations are also described there - make sure to check it out as well.  

## Integration with [cairo-coverage](https://github.com/software-mansion/cairo-coverage)

`snforge` is able to produce a file with a trace for each passing test (excluding fuzz tests).
All you have to do is use the [`--save-trace-data`](../appendix/snforge/test.md#--save-trace-data) flag:

```shell
$ snforge test --save-trace-data
```

The files with traces will be saved to `snfoundry_trace` directory. Each one of these files can then be used as an input
for the [cairo-coverage](https://github.com/software-mansion/cairo-coverage).

If you want `snforge` to call `cairo-coverage` on generated files automatically, use [`--coverage`](../appendix/snforge/test.md#--coverage) flag:

```shell
$ snforge test --coverage
```

This will generate a coverage report in the `coverage` directory named `coverage.lcov`.

## Passing arguments to `cairo-coverage`

You can pass additional arguments to `cairo-coverage` by using the `--` separator. Everything after `--` will be passed
to `cairo-coverage`:

```shell
$ snforge test --coverage -- --include macros
```

> 📝 **Note**
> 
> Running `snforge test --help` won't show info about `cairo-coverage` flags. To see them, run `snforge test --coverage -- --help`.

## Coverage report

`cairo-coverage` generates coverage data as an `.lcov` file. A summary report with aggregated data can be produced by one of many tools that accept the `lcov` format.
In this example we will use the `genhtml` tool from the [lcov package](https://github.com/linux-test-project/lcov/tree/master) to generate an HTML report.

Run the following command in the directory containing your `coverage.lcov` file:

```shell
$ genhtml -o coverage_report coverage.lcov
```

You can now open the `index.html` file in the `coverage_report` directory to see the generated coverage report.