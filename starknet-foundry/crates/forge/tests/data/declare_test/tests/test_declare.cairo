use result::ResultTrait;
use forge_print::PrintTrait;

#[test]
fn test_declare_simple() {
    assert(1 == 1, 'simple check');
    let class_hash = declare('HelloStarknet').unwrap();
    assert(class_hash != 0, 'proper class hash');
}

#[test]
fn multiple_contracts() {
    let class_hash = declare('HelloStarknet').unwrap();
    assert(class_hash != 0, 'proper class hash');

    let class_hash2 = declare('Contract1').unwrap();
    assert(class_hash2 != 0, 'proper class hash');

    assert(class_hash != class_hash2, 'class hashes neq');
}

#[test]
fn non_existent_contract() {
    let class_hash = declare('GoodbyeStarknet').unwrap();
}
