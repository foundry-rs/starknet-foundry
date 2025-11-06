use snforge_std::testing::get_current_vm_step;
use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};
use testing_reference::{ICounterSafeDispatcher, ICounterSafeDispatcherTrait};

#[feature("safe_dispatcher")]
fn setup() {
    // Deploy contract
    let (contract_address, _) = declare("Counter")
        .unwrap()
        .contract_class()
        .deploy(@array![])
        .unwrap();

    let dispatcher = ICounterSafeDispatcher { contract_address };

    // Increment counter a few times
    dispatcher.increment();
    dispatcher.increment();
    dispatcher.increment();
}

#[test]
fn test_setup_steps() {
    let steps_start = get_current_vm_step();
    setup();
    let steps_end = get_current_vm_step();

    // Assert that setup used no more than 100 steps
    assert!(steps_end - steps_start <= 100);
}
