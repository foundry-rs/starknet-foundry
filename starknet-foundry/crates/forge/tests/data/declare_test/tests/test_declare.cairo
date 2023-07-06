use result::ResultTrait;
use protostar_print::PrintTrait;

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
