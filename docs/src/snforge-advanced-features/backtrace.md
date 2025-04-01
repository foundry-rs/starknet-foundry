# Backtrace

## Prerequisites

Backtrace feature relies on debug information provided by Scarb. To generate the necessary debug information, you need
to have:

1. [Scarb](https://github.com/software-mansion/scarb) version `2.8.0` or higher
2. `Scarb.toml` file with the following Cairo compiler configuration:

> 📝 **Note**
> 
> If you are using `scarb nightly-2025-03-27` there is a way to improve backtrace for panic in contracts if you set `panic-backtrace` to true in Scarb.toml

```toml
[profile.dev.cairo]
unstable-add-statements-code-locations-debug-info = true
unstable-add-statements-functions-debug-info = true
panic-backtrace = true # only for scarb nightly-2025-03-27
```

> 📝 **Note**
>
> That `unstable-add-statements-code-locations-debug-info = true` and
`unstable-add-statements-functions-debug-info = true` will slow down the compilation and cause it to use more system
> memory. It will also make the compilation artifacts larger. So it is only recommended to add these flags when you need
> their functionality.

## Usage

When a contract call fails, the error message alone may not always provide enough information to identify the root cause
of the issue. To aid in debugging, `snforge` offers a feature that can generate a backtrace of the execution.

If your contract fails and a backtrace can be generated, `snforge` will prompt you to run the operation again with the
`SNFORGE_BACKTRACE=1` environment variable (if it’s not already configured). For example, you may see failure data like
this:


<!-- { "package_name": "backtrace_vm_error" } -->
```shell
$ snforge test
```
<details>
<summary>Output:</summary>

```shell
Failure data:
    (0x454e545259504f494e545f4e4f545f464f554e44 ('ENTRYPOINT_NOT_FOUND'), 0x454e545259504f494e545f4641494c4544 ('ENTRYPOINT_FAILED'))
note: run with `SNFORGE_BACKTRACE=1` environment variable to display a backtrace
```
</details>
<br>


To enable backtraces, simply set the `SNFORGE_BACKTRACE=1` environment variable and rerun the operation.

When enabled, the backtrace will display the call tree of the execution, including the specific line numbers in the
contracts where the errors occurred. Here's an example of what you might see:

<!-- TODO(#2713) -->

<!-- { "ignored": true, "package_name": "backtrace_vm_error" } -->
```shell
$ SNFORGE_BACKTRACE=1 snforge test
```
<details>
<summary>Output:</summary>

```shell
Failure data:
    (0x454e545259504f494e545f4e4f545f464f554e44 ('ENTRYPOINT_NOT_FOUND'), 0x454e545259504f494e545f4641494c4544 ('ENTRYPOINT_FAILED'))
    
Error occurred in contract 'InnerContract' at pc: '72'
Stack backtrace:
   0: backtrace_vm_error::InnerContract::inner_call
       at [..]/src/lib.cairo:47:9
   1: backtrace_vm_error::InnerContract::InnerContract::inner
       at [..]/src/lib.cairo:38:13
   2: backtrace_vm_error::InnerContract::__wrapper__InnerContract__inner
       at [..]/src/lib.cairo:37:9

Error occurred in contract 'OuterContract' at pc: '107'
Stack backtrace:
   0: backtrace_vm_error::IInnerContractDispatcherImpl::inner
       at [..]/src/lib.cairo:22:1
   1: backtrace_vm_error::OuterContract::OuterContract::outer
       at [..]/src/lib.cairo:17:13
   2: backtrace_vm_error::OuterContract::__wrapper__OuterContract__outer
       at [..]/src/lib.cairo:15:9
```
</details>
<br>

