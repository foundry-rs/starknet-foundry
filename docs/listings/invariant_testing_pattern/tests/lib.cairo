use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};
use invariant_testing_pattern::{IVaultDispatcher, IVaultDispatcherTrait};

// Stateful invariant pattern (recipe).
//
// The fuzzer generates a u256 seed. The test slices the seed into per-step
// action bytes, walks a call sequence against the contract under test, and
// asserts an invariant between every transition.
//
// Invariant: the contract's reported balance equals the deposits-minus-
// withdrawals the test has applied so far. A regression on either branch
// (e.g. withdraw decrementing by amount + 1) drifts the ledger and is
// caught within a few seeds.

const STEPS: u32 = 16;

#[test]
#[fuzzer(runs: 256)]
fn invariant_vault_ledger_matches_contract(seed: u256) {
    let contract = declare("Vault").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let dispatcher = IVaultDispatcher { contract_address };

    let mut remaining: u256 = seed;
    let mut ledger: u64 = 0;
    let mut step: u32 = 0;

    while step < STEPS {
        let action_byte: u64 = (remaining & 0xff).try_into().unwrap();
        remaining = remaining / 256;

        // Cap the amount well below u64::MAX to keep the test ledger arithmetic
        // free of overflow; the pattern is independent of this clamp.
        let amount: u64 = action_byte / 4;
        let is_deposit: bool = (action_byte % 2) == 0;

        if is_deposit {
            dispatcher.deposit(amount);
            ledger = ledger + amount;
        } else if ledger >= amount {
            dispatcher.withdraw(amount);
            ledger = ledger - amount;
        }
        // No-op when the withdraw would underflow the ledger; the contract
        // also rejects that case, so skipping keeps both sides in sync.

        // Invariant — runs after every transition.
        assert!(
            dispatcher.balance() == ledger,
            "vault balance drifted from ledger after step",
        );

        step = step + 1;
    }
}
