use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the transaction account deployment data for the given contract address and span.
/// - `contract_address` - instance of `ContractAddress` specifying which contracts to cheat
/// - `account_contract_address` - transaction account deployment data to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat applied
fn cheat_account_contract_address(
    contract_address: ContractAddress, account_contract_address: ContractAddress, span: CheatSpan
) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .account_contract_address =
            Operation::Start(
                CheatArguments { value: account_contract_address, span, target: contract_address, }
            );

    cheat_execution_info(execution_info);
}

/// Changes the transaction account deployment data.
/// - `account_contract_address` - transaction account deployment data to be set
fn cheat_account_contract_address_global(account_contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .account_contract_address = Operation::StartGlobal(account_contract_address);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_account_contract_address_global`.
fn stop_cheat_account_contract_address_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.account_contract_address = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction account deployment data for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `account_contract_address` - transaction account deployment data to be set
fn start_cheat_account_contract_address(
    contract_address: ContractAddress, account_contract_address: ContractAddress
) {
    cheat_account_contract_address(
        contract_address, account_contract_address, CheatSpan::Indefinite
    );
}

/// Cancels the `cheat_account_contract_address` / `start_cheat_account_contract_address` for the
/// given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to stop cheating
fn stop_cheat_account_contract_address(contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.account_contract_address = Operation::Stop(contract_address);

    cheat_execution_info(execution_info);
}
