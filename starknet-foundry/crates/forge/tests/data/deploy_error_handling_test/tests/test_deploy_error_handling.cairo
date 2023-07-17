use result::ResultTrait;
use cheatcodes::RevertedTransactionTrait;
use cheatcodes::PreparedContract;
use array::ArrayTrait;

#[test]
fn test_deploy_error_handling() {
    let class_hash = declare('PanickingConstructor').expect('Could not declare');
    let prepared_contract = PreparedContract {
        class_hash: class_hash,
        constructor_calldata: @ArrayTrait::new()
    };

    match deploy(prepared_contract) {
        Result::Ok(_) => panic_with_felt252('Should have panicked'),
        Result::Err(x) => {
            assert(*x.panic_data.at(0_usize) == 'PANIK', *x.panic_data.at(0_usize));
            assert(*x.panic_data.at(1_usize) == 'DEJTA', *x.panic_data.at(1_usize));
        }
    }
}
