# Profiling

Profiling is what allows developers to get more insight into how the transaction is executed.
You can inspect the call tree, see how many resources are used for different parts of the execution and more!

## Integration with [cairo-profiler](https://github.com/software-mansion/cairo-profiler)

`snforge` is able to produce a file with a trace for each passing test (excluding fuzz tests). 
All you have to do is use the [`--save-trace-data`](../appendix/snforge/test.md#--save-trace-data) flag:

```shell
$ snforge test --save-trace-data
```

Each one of these files can then be used as an input
for the [cairo-profiler](https://github.com/software-mansion/cairo-profiler).

If you want `snforge` to call `cairo-profiler` on generated files automatically, run:

```shell
$ snforge test --build-profile
``` 
