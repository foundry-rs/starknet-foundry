//ANCHOR:first_half
use snforge_std::{declare, ContractClassTrait, DeclareResultTrait};

use testing_smart_contracts_handling_errors::{
    IPanicContractDispatcher, IPanicContractDispatcherTrait,
};

#[test]
//ANCHOR_END:first_half
//ANCHOR:second_half
fn failing() {
    let contract = declare("PanicContract").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let dispatcher = IPanicContractDispatcher { contract_address };

    dispatcher.do_a_panic();
}
//ANCHOR_END:second_half

mod dummy {} // trick `scarb fmt --check`
