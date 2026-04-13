use crate::{
    predeployment::erc20::contracts_data::load_erc20_predeployed_contracts,
    runtime_extensions::forge_runtime_extension::contracts_data::ContractsData,
};
use anyhow::Result;

pub fn load_predeployed_contracts() -> Result<ContractsData> {
    // In the future, if we have more contracts to predeploy, we can split them into separate functions and merge the results here.
    let erc20_contracts = load_erc20_predeployed_contracts()?;
    ContractsData::try_from(erc20_contracts)
}
