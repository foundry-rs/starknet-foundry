# sncast interface overhaul

## Context
In light of needed changes on how the scripts and contract declaration work (ie adding support for scarb workspaces and various directory layouts),
we need to re-think how we call the respective subcommands, and try to improve it. This design doc covers only a portion
(although significant one) of sncast interface - it was decided recently we should make the interfaces as 'clap-native'
as possible, but this design doc won't cover those changes in detail, as they are pretty straight forward and already have
an opened issue.

## The 'now'
Currently, when declaring a contract we require user to pass the contract name using `--contract-name` flag. This flag only
takes the contract's name, so for example it is not possible to specify the package this contract was defined in (only
achievable when passing `--path-to-scarb-toml`). 

When running a script we require a positional argument, that is the script package name. Similarly to declare, we could
pass `--path-to-scarb-toml` if we wanted to pick the manifest we wanted to use.

These approaches are a bit different to each other, and have a common flaw in the shape of having to specify a manifest 
file one wants to work with, when the package needs to be specified. This may seem quite quirky, as we probably could 
assume that we're always in some workspace/package.

## Proposed changes
We could streamline the way we interact with scarb-related subcommands (declare, script at the moment of writing this)
to look like this:

```bash
$ # this is the same module
$ sncast script path::to::script_module
$ sncast script to::script_module
$ sncast script script_module

$ # this is the same contract
$ sncast declare path::to::contract
$ sncast declare to::contract
$ sncast declare contract

$ # contract is ambiguous - there are multiple contracts named 'contract' in multiple packages
$ sncast --package packagea declare contract
```

In case there is only one script module/contract with specified name in workspace, only this name can be used to target it.
Otherwise, if it is ambiguous in the context of workspace, we should show all matches with full paths (along with package names),
to let user decide which one to use. The `--package` flag would then be required, and would indicate which package to use.

This change would mean that from now on we will have unified interface for declare/script, always assume we're in some workspace/package, 
the `--path-to-scarb-toml` flag can be removed.

## Implementation
As much things as possible should be imported from scarb-api crate. The mechanism for path matching could use scarb's 
package filter mechanism underneath, but probably shouldn't be exposed to the cast interface.
