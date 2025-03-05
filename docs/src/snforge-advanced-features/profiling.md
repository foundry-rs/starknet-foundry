# Profiling

Profiling is what allows developers to get more insight into how the transaction is executed.
You can inspect the call tree, see how many resources are used for different parts of the execution, and more!

## Integration with [cairo-profiler](https://github.com/software-mansion/cairo-profiler)

`snforge` is able to produce a file with a trace for each passing test (excluding fuzz tests). 
All you have to do is use the [`--save-trace-data`](../appendix/snforge/test.md#--save-trace-data) flag:

```shell
$ snforge test --save-trace-data
```

> 💡 **Tip**
>
> You can choose which resource to track (cairo-steps or sierra-gas) using `--tracked-resource` flag
> Tracking sierra gas is only available for sierra 1.7.0+

The files with traces will be saved to `snfoundry_trace` directory. Each one of these files can then be used as an input
for the [cairo-profiler](https://github.com/software-mansion/cairo-profiler).

If you want `snforge` to call `cairo-profiler` on generated files automatically, use [`--build-profile`](../appendix/snforge/test.md#--build-profile) flag:

```shell
$ snforge test --build-profile
``` 
The files with profiling data will be saved to `profile` directory.

## Passing arguments to `cairo-profiler`

You can pass additional arguments to `cairo-profiler` by using the `--` separator. Everything after `--` will be passed
to `cairo-profiler`:

```shell
$ snforge test --build-profile -- --show-inlined-functions
```

> 📝 **Note**
>
> Running `snforge test --help` won't show info about `cairo-profiler` flags. To see them, run `snforge test --build-profile -- --help`.
