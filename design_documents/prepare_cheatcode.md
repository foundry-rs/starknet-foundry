# `prepare` Cheatcode

## Context

Some testing cases require knowing the address of the contract that will be deployed in advance.
These include:

- Using cheatcodes in contract's constructor: This requires running the cheatcode before `deploy` is called.
- Tests that depend on knowing the address of the contract that will be deployed.

Since `deploy` already performs a deployment of the contract identified just by `class_hash` it is impossible to know
the address in advance with the current cheatcodes.

## Goal

Propose a solution that will allow knowing the address of a contract before the deployment.

## Considered Solutions

### Require Calling the `prepare` Cheatcode Before Every Deployment

This is similar to how the contract deployment worked in Protostar:

1. Call `declare` with the contract name. This returns `class_hash`.
2. Call `prepare` with the `class_hash` and `calldata`. This returns `PreparedContract` struct.
3. Call `deploy` with the `PreparedContract` struct. This returns the address.

With this approach, `PreparedContract` also included a `contract_address` field that could be used for applying the
cheatcodes before the contract was deployed.

The problem with this approach is that it is very verbose and requires multiple steps:
Just deploying the contracts is more frequent use case than applying cheatcodes before the deployment.
Users would have to perform an often unnecessary extra step with every deployment.

### Introducing the `precalculate_address` Cheatcode

Introducing the `precalculate_address` cheatcode that would return the contract address of the contract that would be
deployed with `deploy` cheatcode.
The problem with this approach is that for it to work, contract addresses would have to be deterministic.
This would limit the user to only deploying a one instance of the given contact.

## Proposed Solution

Improving on the solution [proposed here](#introducing-the-precalculateaddress-cheatcode), allow precalculating the
address while having the contract address semi-deterministic.

### Salt "Counter"

Introduce an internal "counter" and use its value to salt the otherwise deterministic contract address.
Every time the `deploy` is called, increment this counter, so subsequent calls of `deploy` with the same `class_hash`
will yield different addresses.

### `precalculate_address` Cheatcode

Introduce a cheatcode with the signature:

```cairo
fn precalculate_address(prepared_contract: PreparedContract) -> ContractAddress;
```

That will use the same method of calculating the contract address as `deploy` uses, utilizing the internal counter.
This way the user will have an ability to know the address of the contract that will be deployed, and the current
deployment flow will remain unchanged.

### Known Problems With This Solution

For the address returned by `precalculate_address` to match the address from `deploy`, the user will have to
call `precalculate_address` immediately before the deployment or at least before any other calls to `deploy` as the
internal counter will then be incremented.

This could be remedied by having separate counters for all `class_hashe`es, but it will still remain a limiting factor.

## Example Usage

```cairo
mod HelloStarknet {
    // ...
    
    #[constructor]
    fn constructor(ref self: ContractState) {
        let caller_address = starknet::get_caller_address();
        self.owner.write(caller_address);
    }
}

#[test]
fn call_and_invoke() {
    // Declare the contract
    let class_hash = declare('HelloStarknet');
    
    // Prepare contract for deployment
    let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
    
    // Precalculate the address
    let contract_address = precalulucate_address(prepared);
    
    // Prank the address
    start_prank(contract_address, 1234.into());
    
    // Deploy with pranked constructor
    let contract_address = deploy(prepared).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };
}
```

In case the user does not need to apply cheatcodes to the constructor, the deployment flow remains as before, without
any steps introduced.

```cairo
#[test]
fn call_and_invoke() {
    // Declare the contract
    let class_hash = declare('HelloStarknet');
    
    // Prepare contract for deployment
    let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
        
    // Deploy with pranked constructor
    let contract_address = deploy(prepared).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };
}
```