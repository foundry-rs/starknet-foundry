// Necessary utility function import
use snforge_std::byte_array::try_deserialize_bytearray_error;
use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};
use testing_smart_contracts_handling_errors::{
    IPanicContractSafeDispatcher, IPanicContractSafeDispatcherTrait,
};

#[test]
#[feature("safe_dispatcher")]
fn handling_string_errors() {
    let contract = declare("PanicContract").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let safe_dispatcher = IPanicContractSafeDispatcher { contract_address };

    match safe_dispatcher.do_a_string_panic() {
        Result::Ok(_) => panic!("Entrypoint did not panic"),
        Result::Err(panic_data) => {
            let str_err = try_deserialize_bytearray_error(panic_data.span()).expect('wrong format');

            assert(
                str_err == "This is panicking with a string, which can be longer than 31 characters",
                'wrong string received',
            );
        },
    };
}
