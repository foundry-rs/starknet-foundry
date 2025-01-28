use sncast_std::{
    get_nonce, deploy, DeployResult, ScriptCommandError, ProviderError, StarknetError, FeeSettings,
    TransactionExecutionErrorData
};

use starknet::{ClassHash, Felt252TryIntoClassHash};
use traits::Into;

fn main() {
    let max_fee = 99999999999999999;
    let salt = 0x3;

    let class_hash: ClassHash = 0x059426c817fb8103edebdbf1712fa084c6744b2829db9c62d1ea4dce14ee6ded
        .try_into()
        .expect('Invalid class hash value');

    let deploy_nonce = get_nonce('pending') + 100;
    let deploy_result = deploy(
        class_hash,
        array![0x2, 0x2, 0x0],
        Option::Some(salt),
        true,
        FeeSettings {
            max_fee: Option::Some(max_fee), max_gas: Option::None, max_gas_unit_price: Option::None
        },
        Option::Some(deploy_nonce)
    )
        .unwrap_err();

    println!("{:?}", deploy_result);

    assert(
        ScriptCommandError::ProviderError(
            ProviderError::StarknetError(StarknetError::InvalidTransactionNonce)
        ) == deploy_result,
        'ohno'
    )
}
