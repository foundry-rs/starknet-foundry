use testing_contract_internals::using_library_calls::{
    ILibraryContractSafeLibraryDispatcher, ILibraryContractSafeDispatcherTrait
};
use starknet::{ClassHash, ContractAddress, syscalls::library_call_syscall};
use snforge_std::{declare, DeclareResultTrait};

#[test]
fn test_library_calls() {
    let class_hash = declare("LibraryContract").unwrap().contract_class().class_hash.clone();
    let lib_dispatcher = ILibraryContractSafeLibraryDispatcher { class_hash };

    let value = lib_dispatcher.get_value().unwrap();
    assert(value == 0, 'Incorrect state');

    lib_dispatcher.set_value(10).unwrap();

    let value = lib_dispatcher.get_value().unwrap();
    assert(value == 10, 'Incorrect state');
}
