use sncast_std::{invoke, InvokeResult, ScriptCommandError, RPCError, StarknetError, ScriptCommandErrorTrait};
use starknet::{ContractAddress, Felt252TryIntoContractAddress};
use traits::Into;

fn main() {
    let map_contract_address = 0x1e52f6ebc3e594d2a6dc2a0d7d193cb50144cfdfb7fdd9519135c29b67e427
        .try_into()
        .expect('Invalid contract address value');

    let invoke_result = invoke(map_contract_address, 'put', array![0x10], Option::None, Option::None).unwrap_err();
    invoke_result.print();

    assert(
        ScriptCommandError::RPCError(
            RPCError::StarknetError(StarknetError::ContractError)
        ) == invoke_result,
        'ohno'
    )
}

