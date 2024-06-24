use crate::{cheatcodes::test_environment::TestEnvironment, common::get_contracts};

#[test]
fn precalculate_address_simple() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("HelloStarknet", &contracts_data);

    let precalculated1 = test_env.precalculate_address(&class_hash, &[]);
    let actual1 = test_env.deploy_wrapper(&class_hash, &[]);

    let precalculated2 = test_env.precalculate_address(&class_hash, &[]);
    let actual2 = test_env.deploy_wrapper(&class_hash, &[]);

    assert_eq!(precalculated1, actual1);
    assert_eq!(precalculated2, actual2);
    assert_ne!(actual1, actual2);
}

#[test]
fn precalculate_address_calldata() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("ConstructorSimple", &contracts_data);

    let precalculated1 = test_env.precalculate_address(&class_hash, &[111]);
    let precalculated2 = test_env.precalculate_address(&class_hash, &[222]);

    let actual1 = test_env.deploy_wrapper(&class_hash, &[111.into()]);

    let precalculated2_post_deploy = test_env.precalculate_address(&class_hash, &[222]);

    let actual2 = test_env.deploy_wrapper(&class_hash, &[222.into()]);

    assert_eq!(precalculated1, actual1);
    assert_ne!(precalculated1, precalculated2);
    assert_ne!(precalculated2, precalculated2_post_deploy);
    assert_eq!(precalculated2_post_deploy, actual2);
}
