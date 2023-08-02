use crate::integration::common::corelib::{corelib, predeployed_contracts};
use crate::integration::common::runner::Contract;
use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;

#[test]
fn timestamp_doesnt_decrease_between_transactions() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use cheatcodes::PreparedContract;

            #[starknet::interface]
                trait ITimestamper<TContractState> {
                    fn write_timestamp(ref self: TContractState);
                    fn read_timestamp(self: @TContractState) -> u64;
                }

            #[test]
            fn timestamp_doesnt_decrease() {
                let class_hash = declare('Timestamper');
                let prepared = PreparedContract { class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let dispatcher = ITimestamperDispatcher { contract_address };

                dispatcher.write_timestamp();
                let timestamp = dispatcher.read_timestamp();

                dispatcher.write_timestamp();
                let next_timestamp = dispatcher.read_timestamp();

                assert(next_timestamp >= timestamp, 'timestamp decreases');
            }
    "#
        ),
        Contract::new(
            "Timestamper",
            indoc!(
                r#"
                #[starknet::interface]
                trait ITimestamper<TContractState> {
                    fn write_timestamp(ref self: TContractState);
                    fn read_timestamp(self: @TContractState) -> u64;
                }
                
                #[starknet::contract]
                mod Timestamper {
                    use array::ArrayTrait;
                    use starknet::get_block_timestamp;
                
                    #[storage]
                    struct Storage {
                        time: u64,
                    }
                
                    #[external(v0)]
                    impl ITimestamperImpl of super::ITimestamper<ContractState> {
                        fn write_timestamp(ref self: ContractState) {
                            let time = get_block_timestamp();
                            self.time.write(time);
                        }
                
                        fn read_timestamp(self: @ContractState) -> u64 {
                            self.time.read()
                        }
                    }
                }
    "#
            )
        )
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

#[test]
fn block_doesnt_decrease_between_transactions() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use cheatcodes::PreparedContract;

            #[starknet::interface]
            trait IBlocker<TContractState> {
                fn write_block(ref self: TContractState);
                fn read_block_number(self: @TContractState) -> u64;
                fn read_block_timestamp(self: @TContractState) -> u64;
                fn read_sequencer_address(self: @TContractState) -> ContractAddress;
            }

            #[test]
            fn block_doesnt_decrease() {
               let class_hash = declare('Blocker');
                let prepared = PreparedContract { class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let dispatcher = IBlockerDispatcher { contract_address };

                dispatcher.write_block();
                let block_number = dispatcher.read_block_number();
                let block_timestamp = dispatcher.read_block_timestamp();
                let sequencer_address = dispatcher.read_sequencer_address();

                dispatcher.write_block();
                let next_block_number = dispatcher.read_block_number();
                let next_block_timestamp = dispatcher.read_block_timestamp();
                let next_sequencer_address = dispatcher.read_sequencer_address();

                assert(next_block_number >= block_number, 'block number decreases');
                assert(next_block_timestamp >= block_timestamp, 'block timestamp decreases');
                assert(sequencer_address == next_sequencer_address, 'sequencer changed');
            }
    "#
        ),
        Contract::new(
            "Blocker",
            indoc!(
                r#"
                use starknet::ContractAddress;

                #[starknet::interface]
                trait IBlocker<TContractState> {
                    fn write_block(ref self: TContractState);
                    fn read_block_number(self: @TContractState) -> u64;
                    fn read_block_timestamp(self: @TContractState) -> u64;
                    fn read_sequencer_address(self: @TContractState) -> ContractAddress;
                }

                #[starknet::contract]
                mod Blocker {
                    use array::ArrayTrait;
                    use starknet::get_block_info;
                    use box::BoxTrait;
                    use starknet::ContractAddress;

                    #[storage]
                    struct Storage {
                        block_number: u64,
                        block_timestamp: u64,
                        sequencer_address: ContractAddress,
                    }

                    #[external(v0)]
                    impl IBlockerImpl of super::IBlocker<ContractState> {
                        fn write_block(ref self: ContractState) {
                            let block_info = get_block_info().unbox();
                            self.block_number.write(block_info.block_number);
                            self.block_timestamp.write(block_info.block_timestamp);
                            self.sequencer_address.write(block_info.sequencer_address);
                        }

                        fn read_block_number(self: @ContractState) -> u64 {
                            self.block_number.read()
                        }
                        fn read_block_timestamp(self: @ContractState) -> u64 {
                            self.block_timestamp.read()
                        }
                        fn read_sequencer_address(self: @ContractState) -> ContractAddress {
                            self.sequencer_address.read()
                        }
                    }
                }
    "#
            )
        )
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

// TODO(#292) Make nonce behavior consistent with Starknet
#[ignore]
#[test]
fn nonce_increases_between_transactions() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use cheatcodes::PreparedContract;

            #[starknet::interface]
            trait INoncer<TContractState> {
                fn write_nonce(ref self: TContractState);
                fn read_nonce(self: @TContractState) -> felt252;
            }

            #[test]
            fn nonce_increases_between_transactions() {
                let class_hash = declare('Noncer').unwrap();
                let prepared = PreparedContract { class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let dispatcher = INoncerDispatcher { contract_address };

                dispatcher.write_nonce();
                let nonce = dispatcher.read_nonce();

                dispatcher.write_nonce();
                let next_nonce = dispatcher.read_nonce();

                assert(next_nonce == nonce + 1, 'nonce doesnt increase');
            }
    "#
        ),
        Contract::new(
            "Noncer",
            indoc!(
                r#"
                #[starknet::interface]
                trait INoncer<TContractState> {
                    fn write_nonce(ref self: TContractState);
                    fn read_nonce(self: @TContractState) -> felt252;
                }

                #[starknet::contract]
                mod Noncer {
                    use array::ArrayTrait;
                    use starknet::get_tx_info;
                    use box::BoxTrait;

                    #[storage]
                    struct Storage {
                        nonce: felt252,
                    }

                    #[external(v0)]
                    impl INoncerImpl of super::INoncer<ContractState> {
                        fn write_nonce(ref self: ContractState) {
                            let tx_info = get_tx_info().unbox();
                            let nonce = tx_info.nonce;
                            self.nonce.write(nonce);
                        }

                        fn read_nonce(self: @ContractState) -> felt252 {
                            self.nonce.read()
                        }
                    }
                }
    "#
            )
        )
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

// TODO(#292) Make nonce behavior consistent with Starknet
#[ignore]
#[allow(clippy::too_many_lines)]
#[test]
fn nonce_increases_between_deploys_and_declares() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use cheatcodes::PreparedContract;

            #[starknet::interface]
            trait INoncer<TContractState> {
                fn write_nonce(ref self: TContractState);
                fn read_nonce(self: @TContractState) -> felt252;
            }

            #[test]
            fn nonce_increases_between_transactions() {
                let class_hash = declare('Noncer').unwrap();
                let prepared = PreparedContract { class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let dispatcher = INoncerDispatcher { contract_address };

                dispatcher.write_nonce();
                let nonce = dispatcher.read_nonce();

                let class_hash1 = declare('Contract1').unwrap();
                let class_hash2 = declare('Contract2').unwrap();

                dispatcher.write_nonce();
                let new_nonce = dispatcher.read_nonce();

                assert(new_nonce == nonce + 3, 'nonce doesnt increase');

                let prepared1 = PreparedContract { class_hash: class_hash1, constructor_calldata: @ArrayTrait::new() };
                let prepared2 = PreparedContract { class_hash: class_hash2, constructor_calldata: @ArrayTrait::new() };
                deploy(prepared1).unwrap();
                deploy(prepared2).unwrap();

                dispatcher.write_nonce();
                let new_new_nonce = dispatcher.read_nonce();

                assert(new_new_nonce == new_nonce + 3, 'nonce doesnt increase');
            }
    "#
        ),
        Contract::new(
            "Noncer",
            indoc!(
                r#"
                #[starknet::interface]
                trait INoncer<TContractState> {
                    fn write_nonce(ref self: TContractState);
                    fn read_nonce(self: @TContractState) -> felt252;
                }

                #[starknet::contract]
                mod Noncer {
                    use array::ArrayTrait;
                    use starknet::get_tx_info;
                    use box::BoxTrait;

                    #[storage]
                    struct Storage {
                        nonce: felt252,
                    }

                    #[external(v0)]
                    impl INoncerImpl of super::INoncer<ContractState> {
                        fn write_nonce(ref self: ContractState) {
                            let tx_info = get_tx_info().unbox();
                            let nonce = tx_info.nonce;
                            self.nonce.write(nonce);
                        }

                        fn read_nonce(self: @ContractState) -> felt252 {
                            self.nonce.read()
                        }
                    }
                }
    "#
            )
        ),
        Contract::new(
            "Contract1",
            indoc!(
                r#"
                #[starknet::contract]
                mod Contract1 {
                    #[storage]
                    struct Storage {}
                }
        "#
            )
        ),
        Contract::new(
            "Contract2",
            indoc!(
                r#"
                #[starknet::contract]
                mod Contract2 {
                    #[storage]
                    struct Storage {}

                    fn get_two(self: @ContractState) -> felt252 {
                        2
                    }
                }
        "#
            )
        )
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}
