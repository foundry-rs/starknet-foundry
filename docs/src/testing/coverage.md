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
