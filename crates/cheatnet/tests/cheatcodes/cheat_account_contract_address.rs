use crate::{cheatcodes::test_environment::TestEnvironment, common::assertions::assert_success};
use cheatnet::state::CheatSpan;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

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
        self.cheatnet_state
            .cheat_account_contract_address(target, account_contract_address, span);
    }

    fn start_cheat_account_contract_address(
        &mut self,
        target: ContractAddress,
        account_contract_address: ContractAddress,
    ) {
        self.cheatnet_state
            .start_cheat_account_contract_address(target, account_contract_address);
    }

    fn stop_cheat_account_contract_address(&mut self, target: ContractAddress) {
        self.cheatnet_state
            .stop_cheat_account_contract_address(target);
    }
}

#[test]
fn cheat_account_contract_address_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatAccountContractAddressChecker", &[]);

    let output = test_env.call_contract(&contract_address, "get_account_contract_address", &[]);
    assert_success(output, &[Felt::from(0)]);
}
