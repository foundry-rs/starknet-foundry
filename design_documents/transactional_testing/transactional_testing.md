# Transactional testing proposal

## Context

Some flows are not currently intuitively testable via standard testing methods, 
like accounts logic (__validate__, __execute__ etc.), transaction rejections,
fees, contract upgrade flows, and other e2e testing procedures.

## Goal

In order to facilitate that, we should consider adding a way to test
the contracts in conditions as close to real networks conditions as possible.

Also, the goal is to make the deployments testable and validation 
of the deployments possible to the user. 

The tests should be also runnable in a variety of environments, depending on the
use case (namely katana, devnet, or testnet). This would increase user's confidence
in the release process.

## Proposed solution

We could consider extending `cast script`, since the functionalities would overlap.
This could be considered as a variant of `cast script`, in a way. 

The extending/differentiating factors would be:

- Extra environment-specific functions (for [katana](https://book.dojoengine.org/toolchain/katana/reference.html#custom-methods)/[devnet](https://github.com/0xSpaceShard/starknet-devnet-rs#dumping--loading) - see docs)
- Additional RPC functions support (receipts, etc.)
- Test-like behavior (fail/pass)
- Idempotent functions, for functionalities that can fail when double-running them (i.e. declare)


## Solution architecture

This is pretty straightforward. We ought to use the existing architecture for hint interception,
provide extra functionality via libraries (one for forge, second one for devnet), and collect the cases from
folders using existing collecting logic. 

Note: Currently, the backend architecture between `cast script` and `forge test` is not common,
and the following effort to implement this architecture should be preceded by making that solution.

![txn_based_testing_arch.png](./txn_based_testing_arch.png)


## Solution analysis

### Pros

- Support of external tech (devnet, katana) via specialized libraries
- Flow close to real starknet transactions, fees, testing how the protocol changes affect contracts
- Ability to test accounts (__validate__, __execute__)
- Ability to test real deployment/interaction scripts in environment close to real one

### Cons
- Keeping up the specialized libraries updated as tech adds new features/changes
- Overall higher maintenance cost
- Test execution speed significantly lower (depending on the used endpoints' throughput)
- Dry-run mode from cast scripts might be sufficient for some use cases (testing deployments)
- Might be confusing to users if not communicated correctly

## Alternative approaches

### 1. Extending the deployment scripts

We could extend the concept of deployment scripts, adding more contextual info for deployment scripts and providing 
the additional functions could work, but we would lack test-like behavior (pass/fail), and it could also be confusing
from the point of user to use the deployment scripts in that way.


### 2. Providing utilities to emulate transactional testing

It's similar to alternative 1, but rather than integrating those concepts into deployment scripts,
we could implement them into snforge itself.
This approach would include trying to include this kind of flow into the current tests.
We could provide utilities to test validation + execution in accounts, and/or add more cheatcodes which you'd use in the 
said scenarios only (submit_txn, call, etc.). This would make for more confusing flow, if not used correctly
(lack of intuitive separation of concerns for std funcs).


