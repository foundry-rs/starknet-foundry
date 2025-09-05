use crate::{cheatcodes::test_environment::TestEnvironment, common::assertions::assert_success};
use cheatnet::state::CheatSpan;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

trait CheatAccountContractAddressTrait {
    fn cheat_account_contract_address(
        &mut self,
        target: ContractAddress,
        new_address: u128,
        span: CheatSpan,
    );
    fn start_cheat_account_contract_address(&mut self, target: ContractAddress, new_address: u128);
    fn stop_cheat_account_contract_address(&mut self, target: ContractAddress);
}

impl CheatAccountContractAddressTrait for TestEnvironment {
    fn cheat_account_contract_address(
        &mut self,
        target: ContractAddress,
        new_address: u128,
        span: CheatSpan,
    ) {
        self.cheatnet_state.cheat_account_contract_address(
            target,
            ContractAddress::from(new_address),
            span,
        );
    }

    fn start_cheat_account_contract_address(&mut self, target: ContractAddress, new_address: u128) {
        self.cheatnet_state
            .start_cheat_account_contract_address(target, ContractAddress::from(new_address));
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

    test_env.start_cheat_account_contract_address(contract_address, 123);

    let output = test_env.call_contract(&contract_address, "get_account_contract_address", &[]);
    assert_success(output, &[Felt::from(123)]);
}
