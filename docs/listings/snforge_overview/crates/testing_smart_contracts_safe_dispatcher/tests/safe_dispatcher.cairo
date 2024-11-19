use snforge_std::{declare, ContractClassTrait, DeclareResultTrait};

use testing_smart_contracts_safe_dispatcher::{
    IPanicContractSafeDispatcher, IPanicContractSafeDispatcherTrait
};

#[test]
#[feature("safe_dispatcher")]
fn handling_errors() {
    let contract = declare("PanicContract").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let safe_dispatcher = IPanicContractSafeDispatcher { contract_address };

    match safe_dispatcher.do_a_panic() {
        Result::Ok(_) => panic!("Entrypoint did not panic"),
        Result::Err(panic_data) => {
            assert(*panic_data.at(0) == 'PANIC', *panic_data.at(0));
            assert(*panic_data.at(1) == 'DAYTAH', *panic_data.at(1));
        }
    };
}
