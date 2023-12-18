use snforge_std::{declare, ContractClassTrait, PrintTrait};
use snforge_std::trace::{get_last_call_trace, DisplayArrayCallEntryPoint};

use trace_info::{ISimpleContractDispatcherTrait, ISimpleContractDispatcher};

#[test]
fn test_trace_print() {
    let contract_address = declare('SimpleContract').deploy(@array![]).unwrap();

    ISimpleContractDispatcher { contract_address }.simple_call(10);

    println!("{}", get_last_call_trace());
}
