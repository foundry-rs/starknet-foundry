use core::result::ResultTrait;
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;

#[test]
fn invalid_module_path() {
    let _contract = declare!(nonexistent::MissingContract).unwrap().contract_class();
}
