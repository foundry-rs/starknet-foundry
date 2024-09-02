# Coverage

Coverage reporting allows developers to gain comprehensive insights into how their code is executed.
With `cairo-coverage`, you can generate a coverage report that can later be analyzed for detailed coverage statistics.

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

> 📝 **Note**
> To generate trace data files, it is required to use [Scarb](https://github.com/software-mansion/scarb) version `2.8.0` or higher and include the following in your `Scarb.toml` file:
> ```toml
> [cairo]
> unstable-add-statements-code-locations-debug-info = true
> unstable-add-statements-functions-debug-info = true
> inlining-strategy = "avoid"
> ```
> For more information about these sections, please refer to the [Scarb documentation](https://docs.swmansion.com/scarb/docs/reference/manifest.html#cairo).