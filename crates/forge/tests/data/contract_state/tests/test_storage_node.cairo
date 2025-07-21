use contract_state::storage_node::{
    StorageNodeContract, IStorageNodeContractDispatcher, IStorageNodeContractDispatcherTrait,
};
use snforge_std::interact_with_state;
use crate::utils::deploy_contract;
use starknet::storage::{StoragePointerWriteAccess, StoragePathEntry};

#[test]
fn test_storage_node() {
    let contract_address = deploy_contract("StorageNodeContract", array![]);
    let dispatcher = IStorageNodeContractDispatcher { contract_address };

    let contract_to_set = 0x123.try_into().unwrap();

    assert(dispatcher.get_description_at(1) == '', 'Incorrect description');
    assert(dispatcher.get_data_at(1, contract_to_set, 10) == "", 'Incorrect data');

    interact_with_state(
        contract_address,
        || {
            let mut state = StorageNodeContract::contract_state_for_testing();

            state.random_data.entry(1).description.write('Lorem Ipsum');
            state.random_data.entry(1).data.entry((contract_to_set, 10)).write("Verba sine sensu");
        },
    );

    assert(dispatcher.get_description_at(1) == 'Lorem Ipsum', 'Incorrect description');
    assert(dispatcher.get_data_at(1, contract_to_set, 10) == "Verba sine sensu", 'Incorrect data');
}
