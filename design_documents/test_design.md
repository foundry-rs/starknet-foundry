# Test design

## Context

Current test design in Starknet Forge has some issues:
- gas estimation is problematic since we have to distinguish between pure Cairo gas
(e.g., creating an array in test) and Starknet gas
- starknet functions like `get_caller_address` are unavailable in tests directly
- starknet system calls like `library_call` are unavailable in tests directly
- testing internal contract functions requires exposing them as `external`
- operating on the contract execution layer while tests being pure Cairo code can create confusion

## Goal

Propose a solution of test design that solves most of the aforementioned issues.

## Proposed Solution

Tests should be contracts (just like in the early days of Protostar).
Basically, the code in test should behave as if it was a function called in an invoke transaction.
This clears up the confusion concerning gas estimation — we just calculate the cost of the whole test 
as it is all executed in the Starknet context. Moreover, starknet functions and system calls now can be made available
in tests directly. Last but not least, the confusion concerning what actually happens when using dispatchers will be cleared:
we just call another contract as if we were in invoke transaction.

## Concerns

We need to clearly communicate the changes to the users and explain to them that Starknet Forge is operating on
execution layer (in fact, account layer) rather than on transaction layer. We need to put an emphasis on the fact
that all transactions are equivalent to `invoke` in tests.
