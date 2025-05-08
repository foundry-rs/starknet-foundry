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


<!-- { "package_name": "backtrace_panic" } -->
```shell
$ snforge test
```
<details>
<summary>Output:</summary>

```shell
Failure data:
    0x417373657274206661696c6564 ('Assert failed')
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
"Failure data:
    0x417373657274206661696c6564 ('Assert failed')
error occurred in contract 'InnerContract'
stack backtrace:
   0: (inlined) core::array::ArrayImpl::append
       at [..]array.cairo:135:9
   1: core::array_inline_macro
       at [..]lib.cairo:364:11
   2: (inlined) core::Felt252PartialEq::eq
       at [..]lib.cairo:231:9
   3: (inlined) backtrace_panic::InnerContract::inner_call
       at [..]traits.cairo:442:10
   4: (inlined) backtrace_panic::InnerContract::InnerContract::inner
       at [..]lib.cairo:40:16
   5: backtrace_panic::InnerContract::__wrapper__InnerContract__inner
       at [..]lib.cairo:35:13

error occurred in contract 'OuterContract'
stack backtrace:
   0: (inlined) backtrace_panic::IInnerContractDispatcherImpl::inner
       at [..]lib.cairo:22:1
   1: (inlined) backtrace_panic::OuterContract::OuterContract::outer
       at [..]lib.cairo:17:13
   2: backtrace_panic::OuterContract::__wrapper__OuterContract__outer
       at [..]lib.cairo:15:9"
```
</details>
<br>

