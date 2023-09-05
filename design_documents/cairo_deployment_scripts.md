# Cairo deployment scripts

## Table of contents

* [Context](#context)
* [Goal](#goal)
* [Considered Solutions](#considered-solution)
  * [sncast commands](#sncast-commands)
  * [sncast config](#sncast-config)
  * [interacting with contract](#interacting-with-contract)
  * [running the script](#running-the-script)
  * [error handling](#error-handling)
  * [idempotency](#idempotency)
  * [example script](#example-script)
  * [example state file](#example-state-file)
  * [miscellaneous](#miscellaneous)
* [Proposed steps and tasks](#proposed-stepstasks)

## Context

There should be a possibility to write a script in cairo, that would enable users to make transactions and send them 
to chain. It should allow to declare and deploy contracts as well as apply state transitions on already deployed
contracts.

## Goal

Propose a solution with an example syntax, that would allow to write deployment scripts in cairo.

## Considered solution

This section is split into smaller subsections describing things we will need to tackle while implementing the solution.

### sncast commands
Specific sncast commands (declare, deploy, account) could be imported as regular functions to the scipts, and called as such.
We must make sure our functions return specific types, that make retrieving essential information easier. At the moment 
we should be in a pretty good shape, for most commands we return specific structs (defined in `cast/src/helpers/response_structs.rs`),
but let's double-check everything necessary is included there, and all importable commands are using such structs.

The commands would have to be implemented in the same manner as forge's cheatcodes.

### sncast config
We should allow for all flags passed to sncast to be propagated to the script. CastConfig should also be importable
and usable from script if someone wishes so, but we should not require it. That means, we might have to review our subcommands
and if some of them requires it directly (`account create` is one), we should try to get rid of this dependency. 

### interacting with contract
We will use contracts dispatchers to be able to call/invoke its functions directly 
(e.g. if contract named `mycontract` has a function `myfunction`, we should be able to just do `mycontract.myfunction();`).
Later on we could also support our subcommands (`invoke`, `call`) to call/invoke contracts without dispatchers.

### running the script
The script would essentially run an entrypoint function - `main`. Inside our script subcommand, we will have to compile 
the cairo script file using `scarb build`, and then run it using `scarb cairo-run`. 
The main function would be required in the deployment script, and we should return an error if it is not found.

In order to build the script, there'll need to be a `Scarb.toml` file present in the working directory. We should either:
- look for a Scarb.toml in the same directory script is (this is scarb's default behaviour - we just must make sure scarb
command is executed from this directory - Command::new(...).current_dir(...) should do); if does not exist we could just create it
- create a hidden Scarb.toml with a custom name (eg `.script_name.toml`) and override path to scarb toml using scarb flag;
if a file does not exist let's just create it

Option 1 is probably more in line with scarb ethos, but the option 2 would probably be more practical for us as it would allow
for multiple Scarb.toml files in one directory, where multiple scripts could reside + Scarb.toml file present in cwd would not
confuse sncast subcommands (eg declare also looks for Scarb.toml in cwd, which could lead to errors/user confusion).

The deployment script could then be run like so:
```bash
$ sncast script /path/to/myscript.s.cairo
```

Log info should be outputted to stdout and optionally written to some log file. 
We could have a flag to suppress outputting to stdout or to output with different formats (int, hex).

By default, the scripts could run in some kind of "dry run mode", meaning all the transactions would be simulated with 
specific rpc calls (like `starknet_simulateTransactions`). An interesting idea would be to research an approach similar 
to cheatnet - fork a runner and test the scripts against it (to be assessed if it is feasible). We could also provide 
a way to show a trace of said transactions if required.

The behaviour would be changed with `--broadcast` flag, that would actually execute needed transactions.

### error handling
If the transaction fails, we should put all the relevant info about it to the state file, output log information and stop
script execution.
If the script fails (for any reason that is not connected to transactions - eg rpc node down), we should output relevant
log information.

In case of failed transaction, in the "output" field in state file we could include an error message. After fixing the issue
we should allow users to just re-run the script - all the previous (succeeded) transactions should not be replayed, the
erroneous transaction should be retried and its output should be put into state file.

### idempotency
At the later stages we will want to have the script to be able to track the on chain state using a state file, which would 
in turn allow the script to be called multiple times without any modifications, to ensure desired state. To achieve that
we must ensure idempotency of calls to the network.

The proposed solution would be to use an external json file, that holds hashes of transactions that were already performed.
While executing a command we could first check this file to see if given transaction exists/succeeded, and based on this
either perform and an action (make a transaction) or skip it and proceed to the next thing in script.

To achieve this there are a few possible solutions:
- adjust our existing cast subcommand functions to handle checking the file
  - pros: 
    - just import a subcommand function and use it without a fuss
    - easy to implement for cast subcommands functions
  - cons:
    - we will have to modify currently existing functions to enable state checking
    - this solution does not work for dispatcher functions

- just try to execute functions, parse the output and compare it with the state file
  - pros:
    - no modifications needed for subcommands functions
    - just import a function and use it
  - cons:
    - kind of brute force solution - we should not have to talk to chain since we have all the necessary info in local file
    - executing dispatcher functions can cause us all kinds of trouble (eg - someone could invoke 'send_eth' function multiple times, where in fact they only wanted to do it once)
    - not every cast subcommand is idempotent

- implement a higher order function (or function wrapper) to conditionally execute functions based on the information in state file
  - pros:
    - no modifications needed to cast subcommands functions
    - works with dispatcher functions just fine
    - added flexibility - user can decide which function should be replayed each time, and which shouldn't
  - cons:
    - boilerplate code - wrapping each function call will be ugly
    - no option to just annotate main function (procedural macros could achieve this, but they do some compile-time modifications)
    - potentially more complex

An example pseudo-implementation of this could look like this:
```rust
fn skip_if_done<F>(func: F)
where
    F: Fn() + 'static,
{
    // Decorator logic here
    // Check the state file and decide whether to execute the function or not

    if should_execute {
        func();
        // write func outputs to state file
    } else {
        // Do nothing or return an appropriate value
    }
}

fn main() {
    skip_if_done(|| {
        dispatcher.send_eth(...);
    });
}
```

- Use a procedural macro to automatically apply decorator-like behaviour to functions, which will decide on whether
given function should be executetd or not (based on state file)
  - pros:
    - elegant solution
    - could be applied selectively (per function) or globally (to main)
  - cons:
    - more complex solution
    - probably not very suitable for runtime based behaviours (like reading json file) - requires more research

An example pseudo-implementation of this could look like this:
```rust
// skip_if_done.rs
use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub use skip_if_done_impl::skip_if_done;


// skip_if_done_impl.rs
use quote::quote;
use std::env;

fn should_execute() -> bool {
    // Your logic to check whether the function should be executed goes here
    true
}

#[macro_export]
macro_rules! skip_if_done {
    ($($tokens:tt)*) => {
        {
            if should_execute() {
                $($tokens)*
            }
        }
    };
}

// main.rs
#[path = "skip_if_done.rs"]
mod skip_if_done;

fn main() {
    skip_if_done! {
        dispatcher.send_eth(...);
    }
}
```

Given all this, the most obvious solutions seem to be:
  - for cast subcommands functions:
    - adjust them to handle checking the file
  - for dispatcher functions:
    - implement a higher order function

Picking those would ensure us that:
  - syntax is relatively boiler-code free (only dispatcher functions are annotated)
  - idempotency is achieved
  - it is quite easy to write a script

### example script
An example deployment script could look like this:

```cairo
// we might need to rename account functions to avoid confusion
use cast::starknet_commands::account::create::create as create_account
use cast::starknet_commands::account::deploy::deploy as deploy_account
use cast::{get_provider, get_account};
use cast::starknet_commands::{declare, deploy, invoke, call};
(...)

fn make_account(provider: &JsonRpcClient<HttpTransport>) -> Result<SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>> {
  let mut prefunder = get_account("user", &provider)?; // the user that will prefund new account 
  let user = create_account("user1", &provider);
  let prefund_account = invoke("0x123", "deposit", [1234, user.address], &prefunder);
  deploy_account("user1", &provider);
  get_account("user1", &provider)?
}

fn main() {
  ler provider = get_provider("http://127.0.0.1:5050/rpc")?;
  let user = make_account(provider);

  let declared_contract = declare("mycontract", &user, "/path/to/Scarb.toml");
  let contract = deploy(&declared_contract.class_hash, [], &user);

  let dispatcher = DispatcherMyContract { contract.contract_address };

  skip_if_done(|| {
        dispatcher.increase_balance(5);
    });
  let get_balance = skip_if_done(|| {
        dispatcher.get_balance();
    });

  let called_balance = call(&contract.contract_address, "get_balance", [], "latest"); 
}
```

### example state file
The state file by default should be written to the current working directory, with a name `<script file name>.state.json`.
Its schema could look like this:

```json
{
  "create_account": {
    "arguments": {
      "name": "whatever",
      (...)
    },
    "output": {
      "address": "0x123",
      "max_fee": "0x321",
    },
    "status": "accepted",
    "timestamp": (...),
  },
  "prefund_account": {
    "arguments": {
      ...
    },
    "output": {
      "error": "..."
    }
  },
  "status": "rejected",
  "timestamp": (...),
}
```

Having this, we could allow users to `--retries` or `--verify` transactions if something unexpected happens,
some transactions were rejected because of faulty script, too low user eth balance etc.

If we were to implement dry run mode that runs by default, the state should only be written to a file when we pass `--broadcast`
flag; otherwise the output should be directed to stdout.

The file should be used to see, if given action/transaction was already taken - it should take under account the function
name as well as its parameters, to distinguish functions from one another (eg. user creates two accounts, we should check
state file records, one by one, and check if a function `account new` was already invoked with specified parameter `username` 
(and others, if supplied)).

### Miscellaneous

There would be a new subcommand `script` that would be invoked with required parameter - a path to script file, with
`*.s.cairo` extension. Proposed interface:

- `--gas-multiplier` - relative percentage by which to multiply add gas estimates (perhaps passed directly to sncast)
- `--broadcast` - broadcasts the transactions
- `--no-state-file` - do not write to state file
- `--state-file` - specify path to custom state file
- `--log-file` - output logs to a file
- `--quiet` - do not output logs to stdout
- `--slow` - makes sure a transaction is sent, only after its previous one has been confirmed and succeeded (possible reuse of `--wait` flag)
- `--verify` - find a matching broadcast in state file, and try to verify transactions
- `--delay` - optional timeout to apply in between attempts in seconds (perhaps passed directly to sncast)
- `--retries` - number of attempts for retrying (perhaps passed directly to sncast)

If it makes sense, some of the flags could be included in Scarb.toml config file.

## Proposed steps/tasks

MVP:
  - allow for writing scripts using dispatchers and imported cast subcommand functions
    - no idempotency required at this stage
    - all cast subcommand functions return every information necessary (tx hashes, addresses etc) as importable structs
    - no cast subcommand requires CastConfig directly (eg as parameter)
    - it is possible to create and deploy an account, create and deploy a contract
    - it is possible to call/invoke contracts using dispatchers
    - `script` subcommand is added to cast, that builds and runs the script
    - logs are outputted to stdout
    - basic tests for `script` subcommand
    - docs

next iteration:
  - invoke, call and multicall (after it is possible to use it without a file [issue 502](https://github.com/foundry-rs/starknet-foundry/issues/502)) subcommands support
  - dry run
  - writing to state file
  - idempotency

next iteration:
  - add support for flags from [Miscellaneous](#miscellaneous)
  - script tests
