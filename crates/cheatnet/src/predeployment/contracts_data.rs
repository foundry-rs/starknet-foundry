use crate::{
    predeployment::erc20::contracts_data::load_erc20_predeployed_contracts,
    predeployment::erc20::eth::{ETH_CONTRACT_CLASS_HASH, ETH_CONTRACT_NAME},
    predeployment::erc20::strk::{STRK_CONTRACT_CLASS_HASH, STRK_CONTRACT_NAME},
    runtime_extensions::forge_runtime_extension::contracts_data::ContractsData,
};
use anyhow::Result;
use conversions::string::TryFromHexStr;

pub fn load_predeployed_contracts() -> Result<ContractsData> {
    // In the future, if we have more contracts to predeploy, we can split them into separate functions and merge the results here.
    let erc20_contracts = load_erc20_predeployed_contracts()?;

    let mut contracts_data = ContractsData::try_from(erc20_contracts)?;

    // Class hashes for STRK and ETH contract are different on network than the ones
    // calculated from their sierras, so we need to update them.
    let class_hashes_to_update = vec![
        (STRK_CONTRACT_NAME.to_string(), STRK_CONTRACT_CLASS_HASH),
        (ETH_CONTRACT_NAME.to_string(), ETH_CONTRACT_CLASS_HASH),
    ];

    for (contract_name, class_hash) in class_hashes_to_update {
        let class_hash = TryFromHexStr::try_from_hex_str(class_hash)?;

        contracts_data
            .class_hashes
            .insert(contract_name.clone(), class_hash);

        // update contract data class hash
        contracts_data
            .contracts
            .get_mut(&contract_name)
            .expect("contract data should be present")
            .class_hash = class_hash;
    }

    Ok(contracts_data)
}
