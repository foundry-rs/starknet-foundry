use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress,
};

/// Changes the address of an account which the transaction originates from, for the given contract
/// address and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat
/// - `account_contract_address` - transaction account deployment data to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat
/// applied
pub fn cheat_account_contract_address(
    target: ContractAddress, account_contract_address: ContractAddress, span: CheatSpan,
) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .account_contract_address =
            Operation::Start(
                CheatArguments { value: account_contract_address, span, target },
            );

    cheat_execution_info(execution_info);
}

/// Changes the address of an account which the transaction originates from.
/// - `account_contract_address` - transaction account deployment data to be set
pub fn start_cheat_account_contract_address_global(account_contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .account_contract_address = Operation::StartGlobal(account_contract_address);

    cheat_execution_info(execution_info);
}

/// Cancels the `start_cheat_account_contract_address_global`.
pub fn stop_cheat_account_contract_address_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.account_contract_address = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the address of an account which the transaction originates from, for the given
/// target contract address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `account_contract_address` - transaction account deployment data to be set
pub fn start_cheat_account_contract_address(
    target: ContractAddress, account_contract_address: ContractAddress,
) {
    cheat_account_contract_address(
        target, account_contract_address, CheatSpan::Indefinite,
    );
}

/// Cancels the `cheat_account_contract_address` / `start_cheat_account_contract_address` for the
/// given target contract address.
/// - `target` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_account_contract_address(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.account_contract_address = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
