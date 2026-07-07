use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};
use trace_resources::sha_checker::{IShaCheckerDispatcher, IShaCheckerDispatcherTrait};

#[test]
fn test_sha() {
    let checker = declare("ShaChecker").unwrap().contract_class();
    let (checker_address, _) = checker.deploy(@array![]).unwrap();

    let dispatcher = IShaCheckerDispatcher { contract_address: checker_address };
    dispatcher.use_sha();
}
