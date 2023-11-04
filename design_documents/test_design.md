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

## Proposed solution

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

For example for a contract:

```cairo
#[starknet::contract]
mod Contract {
    #[storage]
    struct Storage {
        balance: felt252, 
        debt: felt252,
    }
    
    //...
    
    #[generate_trait]
    impl InternalImpl of InternalTrait {
        fn internal_function(self: @ContractState) -> felt252 {
            self.balance.read()
        }
    }
}
```

we can set initial storage values (keep in mind it modifies the storage of the test) and call internal functions like this:

```cairo
use pkg::HelloStarknet::balanceContractMemberStateTrait;
use pkg::HelloStarknet::debtContractMemberStateTrait;

#[test]
fn test_internal() {
    let mut state = HelloStarknet::contract_state_for_testing();

    state.balance.write(10);
    state.debt.write(5);
    
    HelloStarknet::InternalImpl::internal_function(@state);
}
```

## Concerns

We need to clearly communicate the changes to the users and explain to them that Starknet Forge is operating on
execution layer (in fact, account layer) rather than on transaction layer. We need to put an emphasis on the fact
that tests should be treated as if they were happening in an `invoke` transaction.
