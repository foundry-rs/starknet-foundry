# Test design

## Context

Current test design in Starknet Forge has some issues:
1. Gas estimation is problematic since we have to distinguish between pure Cairo gas
(e.g., creating an array in test) and Starknet gas
2. Starknet functions like `get_caller_address` and system calls like `library_call` are unavailable in tests directly 
3. Testing internal contract functions requires exposing them as `external`
4. Operating on the contract execution layer while tests being pure Cairo code can create confusion

## Goal

Propose a solution of test design that solves most of the aforementioned issues.

## Proposed Solution

Tests should be contracts (just like in the early days of Protostar).
Basically, the code in test should behave as if it was a function called in an invoke transaction.

Solutions to aforementioned problems:
1. Since everything happens in the Starknet context now, we can just calculate the cost of the whole test. 
2. Starknet functions and system calls can be made available in tests directly since tests, being contracts, have their own states.
3. See next [section](#testing-internal-functions).
4. The confusion concerning what actually happens when using dispatchers is cleared: we just call another contract as if we were in an invoke transaction.

## Testing internal functions

When implementing contracts, a function called `contract_state_for_testing` is created just before compilation to Sierra/CASM, just like with dispatchers.
It can be used to obtain the state (storage) of the current (to be tested) contract. Since each test is a contract, now internal functions can be tested
using the storage of the test.

```rust
use pkg::HelloStarknet::debtContractMemberStateTrait;
use pkg::HelloStarknet::balanceContractMemberStateTrait;

#[test]
fn test_essa() {
    let mut state = HelloStarknet::contract_state_for_testing();
    state.balance.read();
    state.debt.write(5);
}
```

## Concerns

We need to clearly communicate the changes to the users and explain to them that Starknet Forge is operating on
execution layer (in fact, account layer) rather than on transaction layer. We need to put an emphasis on the fact
that everything that they should treat tests as if they were happening in an `invoke` transaction.
