use cheatnet::state::CheatSpan;
use starknet_api::core::ContractAddress;

use crate::cheatcodes::test_environment::TestEnvironment;

trait CheatAccountContractAddressTrait {
    fn cheat_account_contract_address(
        &mut self,
        target: ContractAddress,
        account_contract_address: ContractAddress,
        span: CheatSpan,
    );
    fn start_cheat_account_contract_address(
        &mut self,
        target: ContractAddress,
        account_contract_address: ContractAddress,
    );
    fn stop_cheat_account_contract_address(&mut self, target: ContractAddress);
}

impl CheatAccountContractAddressTrait for TestEnvironment {
    fn cheat_account_contract_address(
        &mut self,
        target: ContractAddress,
        account_contract_address: ContractAddress,
        span: CheatSpan,
    ) {
        todo!()
    }

    fn start_cheat_account_contract_address(
        &mut self,
        target: ContractAddress,
        account_contract_address: ContractAddress,
    ) {
        todo!()
    }

    fn stop_cheat_account_contract_address(&mut self, target: ContractAddress) {
        todo!()
    }
}
