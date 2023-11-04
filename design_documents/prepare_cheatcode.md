# `prepare` Cheatcode

<!-- TOC -->
* [`prepare` Cheatcode](#prepare-cheatcode)
  * [Context](#context)
  * [Goal](#goal)
  * [Considered Solutions](#considered-solutions)
    * [Require Calling the `prepare` Cheatcode Before Every Deployment](#require-calling-the-prepare-cheatcode-before-every-deployment)
    * [Introducing the `precalculate_address` Cheatcode](#introducing-the-precalculateaddress-cheatcode)
      * [Salt "Counter"](#salt-counter)
      * [`precalculate_address` Cheatcode](#precalculateaddress-cheatcode)
      * [Known Problems With This Solution](#known-problems-with-this-solution)
      * [Example Usage](#example-usage)
  * [Proposed Solution](#proposed-solution)
    * [`declare` Cheatcode](#declare-cheatcode)
    * [Salt "Counter"](#salt-counter-1)
    * [Known Problems With This Solution](#known-problems-with-this-solution-1)
    * [Example Usage](#example-usage-1)
<!-- TOC -->

## Context

Some testing cases require knowing the address of the contract that will be deployed in advance.
These include:

- Using cheatcodes in contract's constructor: This requires running the cheatcode before `deploy` is called.
- Tests that depend on knowing the address of the contract that will be deployed.

Since `deploy` already performs a deployment of the contract identified just by `class_hash` it is impossible to know
the address in advance with the current cheatcodes.

## Goal

Propose a solution that will allow knowing the address of a contract before the deployment.

## Considered solutions

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

#### Salt "counter"

Introduce an internal "counter" and use its value to salt the otherwise deterministic contract address.
Every time the `deploy` is called, increment this counter, so subsequent calls of `deploy` with the same `class_hash`
will yield different addresses.

#### `precalculate_address` Cheatcode

Introduce a cheatcode with the signature:

```cairo
fn precalculate_address(prepared_contract: PreparedContract) -> ContractAddress;
```

That will use the same method of calculating the contract address as `deploy` uses, utilizing the internal counter.
This way the user will have an ability to know the address of the contract that will be deployed, and the current
deployment flow will remain unchanged.

#### Known problems with this solution

For the address returned by `precalculate_address` to match the address from `deploy`, the user will have to
call `precalculate_address` immediately before the deployment or at least before any other calls to `deploy` as the
internal counter will then be incremented.

This could be remedied by having separate counters for all `class_hashe`es, but it will still remain a limiting factor.

#### Example usage

```cairo
mod HelloStarknet {
    // ...
    
    #[constructor]
    fn constructor(ref self: ContractState) {
        let timestamp = starknet::get_block_timestamp();
        self.create_time.write(timestamp);
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
    
    // Warp the address
    start_warp(contract_address, 1234);
    
    // Deploy with warped constructor
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
        
    // Deploy
    let contract_address = deploy(prepared).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };
}
```

## Proposed solution

Change the current deployment flow, so it can better facilitate precalculating of contract addresses.

### `declare` Cheatcode

Change the `declare` cheatcode signature to this:

```cairo
struct ContractClass {
    class_hash: ClassHash
    // ...
}

trait ContractClassTrait {
    fn precalculate_address(self: @ContractClass, constructor_calldata: @Array::<felt252>) -> ContractAddress;
    fn deploy(self: @ContractClass, constructor_calldata: @Array::<felt252>) -> Result::<ContractAddress, RevertedTransaction>;
}

impl ContractClassTrait of ContractClassTrait {
    // ...
}

fn declare(contract: felt252) -> ContractClass;
```

And remove the `deploy` cheatcode entirely.

Both `precalculate_address` and `deploy` should use the same way of calculating the contract address.

### Salt "counter"

Introduce the same salt counter as [discussed here](#salt-counter).
This will allow deterministic address calculation and deploying of multiple instances of the same contract.

### Known problems with this solution

Same problems as [indicated here](#known-problems-with-this-solution) apply to Proposed Solution 2 as well.

### Example usage

```cairo
mod HelloStarknet {
    // ...
    
    #[constructor]
    fn constructor(ref self: ContractState) {
        let timestamp = starknet::get_block_timestamp();
        self.create_time.write(timestamp);
    }
}

#[test]
fn call_and_invoke() {
    // Declare the contract
    let contract = declare('HelloStarknet');
        
    // Precalculate the address
    let contract_address = contract.precalulucate_address(@ArrayTrait::new());
    
    // Warp the address
    start_warp(contract_address, 1234);
    
    // Deploy with warped constructor
    let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };
}
```
