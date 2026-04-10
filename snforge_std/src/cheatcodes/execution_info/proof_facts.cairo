use super::{
    CheatArguments, CheatSpan, ContractAddress, ExecutionInfoMock, Operation, cheat_execution_info,
};

/// Changes the transaction proof facts for the given contract address and span.
/// - `contract_address` - instance of `ContractAddress` specifying which contracts to cheat
/// - `proof_facts` - transaction proof facts to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat
/// applied
pub fn cheat_proof_facts(
    contract_address: ContractAddress, proof_facts: Span<felt252>, span: CheatSpan,
) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .proof_facts =
            Operation::Start(CheatArguments { value: proof_facts, span, target: contract_address });

    cheat_execution_info(execution_info);
}

/// Changes the transaction proof facts.
/// - `proof_facts` - transaction proof facts to be set
pub fn start_cheat_proof_facts_global(proof_facts: Span<felt252>) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.proof_facts = Operation::StartGlobal(proof_facts);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_proof_facts_global`.
pub fn stop_cheat_proof_facts_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.proof_facts = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction proof facts for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `proof_facts` - transaction proof facts to be set
pub fn start_cheat_proof_facts(contract_address: ContractAddress, proof_facts: Span<felt252>) {
    cheat_proof_facts(contract_address, proof_facts, CheatSpan::Indefinite);
}

/// Cancels the `cheat_proof_facts` / `start_cheat_proof_facts` for the
/// given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_proof_facts(contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.proof_facts = Operation::Stop(contract_address);

    cheat_execution_info(execution_info);
}
