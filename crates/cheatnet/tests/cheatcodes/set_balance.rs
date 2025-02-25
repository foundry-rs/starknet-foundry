use super::test_environment::TestEnvironment;
use crate::common::assertions::assert_success;
use crate::common::get_contracts;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallResult;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::erc20::set_balance;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::erc20::Token;
use conversions::felt::FromShortString;
use starknet::core::types::U256;
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

trait SetBalanceTrait {
    fn set_balance(&mut self, target: ContractAddress, new_balance: U256, token: Token);
}

impl SetBalanceTrait for TestEnvironment {
    fn set_balance(&mut self, target: ContractAddress, new_balance: U256, token: Token) {
        set_balance(&mut self.cached_state, target, new_balance, token).unwrap();
    }
}

fn get_balance(
    test_env: &mut TestEnvironment,
    target: ContractAddress,
    token: Token,
) -> CallResult {
    test_env.call_contract(&token.contract_address(), "balance_of", &[target.into()])
}

#[test]
fn test_set_balance_strk() {
    let token = Token::STRK;
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("HelloStarknet", &contracts_data);
    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);

    let balance = get_balance(&mut test_env, contract_address, token);
    assert_success(balance, &[0.into(), 0.into()]);

    test_env.set_balance(contract_address, U256::from(1_000_000_u32), token);
    let balance = get_balance(&mut test_env, contract_address, token);
    assert_success(balance, &[1_000_000.into(), 0.into()]);

    test_env.set_balance(contract_address, U256::from_words(u128::MAX, 100), token);
    let balance = get_balance(&mut test_env, contract_address, token);
    assert_success(balance, &[u128::MAX.into(), 100.into()]);
}

#[test]
fn test_set_balance_custom_token() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("HelloStarknet", &contracts_data);
    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);

    let custom_token_class_hash = test_env.declare("ERC20", &contracts_data);
    let custom_token_address = test_env.deploy_wrapper(
        &custom_token_class_hash,
        &[
            Felt::from_short_string("CustomToken").unwrap(),
            Felt::from_short_string("CT").unwrap(),
            18.into(),
            1_000_000_000.into(),
            0.into(),
            123.into(),
        ],
    );

    let token = Token::Custom {
        contract_address: custom_token_address,
        balances_variable_selector: get_selector_from_name("balances").unwrap(),
    };

    let balance = get_balance(&mut test_env, contract_address, token);
    assert_success(balance, &[0.into(), 0.into()]);

    test_env.set_balance(contract_address, U256::from(1_000_000_u32), token);
    let balance = get_balance(&mut test_env, contract_address, token);
    assert_success(balance, &[1_000_000.into(), 0.into()]);

    test_env.set_balance(contract_address, U256::from_words(u128::MAX, 100), token);
    let balance = get_balance(&mut test_env, contract_address, token);
    assert_success(balance, &[u128::MAX.into(), 100.into()]);
}
