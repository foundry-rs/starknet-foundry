use std::path::Path;

use crate::integration::common::runner::Contract;
use crate::integration::common::running_tests::run_test_case;
use crate::{assert_case_output_contains, assert_failed, assert_passed, test_case};
use indoc::indoc;

#[test]
fn test_storage_access_from_tests() {
    let test = test_case!(indoc!(
        r#"
        #[starknet::contract]
        mod Contract {
            #[storage]
            struct Storage {
                balance: felt252, 
            }
            
            #[generate_trait]
            impl InternalImpl of InternalTrait {
                fn internal_function(self: @ContractState) -> felt252 {
                    self.balance.read()
                }
            }
        }

        use tests::test_case::Contract::balanceContractMemberStateTrait;

        #[test]
        fn test_internal() {
            let mut state = Contract::contract_state_for_testing();
            state.balance.write(10);
            
            let value = Contract::InternalImpl::internal_function(@state);
            assert(value == 10, 'Incorrect storage value');
        }
    "#
    ),);

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn test_simple_syscalls() {
    let test = test_case!(
        indoc!(
            r#"
        use starknet::info::{get_execution_info, TxInfo};
        use result::ResultTrait;
        use box::BoxTrait;
        use serde::Serde;
        use starknet::{ContractAddress, get_block_hash_syscall};
        use array::SpanTrait;
        use snforge_std::{ declare, ContractClassTrait, test_address };

        #[starknet::interface]
        trait ISpoofChecker<TContractState> {
            fn get_tx_hash(ref self: TContractState) -> felt252;
            fn get_nonce(ref self: TContractState) -> felt252;
            fn get_account_contract_address(ref self: TContractState) -> ContractAddress;
            fn get_signature(ref self: TContractState) -> Span<felt252>;
            fn get_version(ref self: TContractState) -> felt252;
            fn get_max_fee(ref self: TContractState) -> u128;
            fn get_chain_id(ref self: TContractState) -> felt252;
        }
        #[starknet::interface]
        trait IRollChecker<TContractState> {
            fn get_block_number(ref self: TContractState) -> u64;
        }

        #[starknet::interface]
        trait IWarpChecker<TContractState> {
            fn get_block_timestamp(ref self: TContractState) -> u64;
        }

     #[starknet::interface]
        trait ISequencerAddressChecker<TContractState> {
            fn get_sequencer_address(self: @TContractState) -> ContractAddress;
        }

        #[test]
        fn test_get_execution_info() {
            let exec_info = get_execution_info().unbox();
            assert(exec_info.caller_address.into() == 0, 'Incorrect caller address');
            assert(exec_info.contract_address == test_address(), exec_info.contract_address.into());
            // Hash of TEST_CASE_SELECTOR
            assert(exec_info.entry_point_selector.into() == 655947323460646800722791151288222075903983590237721746322261907338444055163, 'Incorrect entry point selector');

            let block_info = exec_info.block_info.unbox();

            let contract_roll = declare('RollChecker');
            let contract_address_roll = contract_roll.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher_roll = IRollCheckerDispatcher { contract_address: contract_address_roll };

            let contract_warp = declare('WarpChecker');
            let contract_address_warp = contract_warp.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher_warp = IWarpCheckerDispatcher { contract_address: contract_address_warp };
            
            let contract_sequencer_add = declare('SequencerAddressChecker');
            let contract_sequencer_add_address = contract_sequencer_add.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher_sequencer_add = ISequencerAddressCheckerDispatcher { contract_address: contract_sequencer_add_address };

            assert(dispatcher_roll.get_block_number() == block_info.block_number, 'Invalid block number');
            assert(dispatcher_warp.get_block_timestamp() == block_info.block_timestamp, 'Invalid block timestamp');
            assert(dispatcher_sequencer_add.get_sequencer_address() == block_info.sequencer_address, 'Invalid block timestamp');

            let contract = declare('SpoofChecker');
            let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher = ISpoofCheckerDispatcher { contract_address };

            let tx_info = exec_info.tx_info.unbox();
            assert(tx_info.version == dispatcher.get_version(), 'Incorrect version');
            assert(tx_info.account_contract_address == dispatcher.get_account_contract_address(), 'Incorrect acc_address');
            assert(tx_info.max_fee == dispatcher.get_max_fee(), 'Incorrect max fee');
            assert(tx_info.signature == dispatcher.get_signature(), 'Incorrect signature');
            assert(tx_info.transaction_hash == dispatcher.get_tx_hash(), 'Incorrect transaction_hash');
            assert(tx_info.chain_id == dispatcher.get_chain_id(), 'Incorrect chain_id');
            assert(tx_info.nonce == dispatcher.get_nonce(), 'Incorrect nonce');
        }
    "#
        ),
        Contract::from_code_path(
            "SpoofChecker".to_string(),
            Path::new("tests/data/contracts/spoof_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "RollChecker".to_string(),
            Path::new("tests/data/contracts/roll_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "WarpChecker".to_string(),
            Path::new("tests/data/contracts/warp_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "SequencerAddressChecker".to_string(),
            Path::new("tests/data/contracts/sequencer_address_checker.cairo")
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn test_get_block_hash_syscall() {
    let test = test_case!(
        indoc!(
            r#"
        use starknet::info::{get_execution_info, TxInfo};
        use result::ResultTrait;
        use box::BoxTrait;
        use serde::Serde;
        use starknet::{ContractAddress, get_block_hash_syscall};
        use array::SpanTrait;
        use snforge_std::{ declare, ContractClassTrait, test_address };

        #[starknet::interface]
        trait BlockHashChecker<TContractState> {
            fn write_block(ref self: TContractState);
            fn read_block_hash(self: @TContractState) -> felt252;
        }

        #[test]
        fn test_get_block_hash() {
            let block_hash_checker = declare('BlockHashChecker');
            let block_hash_checker_address = block_hash_checker.deploy(@ArrayTrait::new()).unwrap();
            let block_hash_checker_dispatcher = BlockHashCheckerDispatcher { contract_address: block_hash_checker_address };
            
            block_hash_checker_dispatcher.write_block();
            
            let stored_blk_hash = block_hash_checker_dispatcher.read_block_hash();
            assert(stored_blk_hash == 0, 'Wrong stored blk hash');
        }
    "#
        ),
        Contract::from_code_path(
            "BlockHashChecker".to_string(),
            Path::new("tests/data/contracts/block_hash_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn test_library_calls() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use starknet::{ ClassHash, library_call_syscall, ContractAddress };
        use snforge_std::{ declare };

        #[starknet::interface]
        trait ILibraryContract<TContractState> {
            fn get_value(
                self: @TContractState,
            ) -> felt252;

            fn set_value(
                ref self: TContractState,
                number: felt252
            );
        }

        #[test]
        fn test_get_execution_info() {
            let class_hash = declare('LibraryContract').class_hash;
            let lib_dispatcher = ILibraryContractSafeLibraryDispatcher { class_hash };
            let value = lib_dispatcher.get_value().unwrap();
            assert(value == 0, 'Incorrect state');
            lib_dispatcher.set_value(10);
            let value = lib_dispatcher.get_value().unwrap();
            assert(value == 10, 'Incorrect state');
        }
    "#
        ),
        Contract::new(
            "LibraryContract",
            indoc!(
                r#"
                #[starknet::contract]
                mod LibraryContract {
                    use result::ResultTrait;
                    use starknet::ClassHash;
                    use starknet::library_call_syscall;

                    #[storage]
                    struct Storage {
                        value: felt252
                    }

                    #[external(v0)]
                    fn get_value(
                        self: @ContractState,
                    ) -> felt252 {
                       self.value.read()
                    }

                    #[external(v0)]
                    fn set_value(
                        ref self: ContractState,
                        number: felt252
                    ) {
                       self.value.write(number);
                    }
                }
                "#
            )
        )
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn test_disabled_syscalls() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use starknet::{ClassHash, deploy_syscall, replace_class_syscall, get_block_hash_syscall};
        use snforge_std::declare;
        
        #[test]
        fn test_replace_class() {
            let value : ClassHash = 'xd'.try_into().unwrap();
            replace_class_syscall(value);
        }
    "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(
        result,
        "test_replace_class",
        "Replace class can't be used in tests"
    );
}

#[test]
fn test_get_block_hash() {
    let test = test_case!(indoc!(
        r#"
        use result::ResultTrait;
        use box::BoxTrait;
        use starknet::{get_block_hash_syscall, get_block_info};

        #[test]
        fn test_get_block_hash() {
            let block_info = get_block_info().unbox();
            let hash = get_block_hash_syscall(block_info.block_number - 10).unwrap();
            assert(hash == 0, 'Hash not zero');
        }
    "#
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn test_cant_call_test_contract() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use starknet::{ClassHash, ContractAddress, deploy_syscall, replace_class_syscall, get_block_hash_syscall};
        use snforge_std::{ declare, ContractClassTrait, test_address };

        #[starknet::interface]
        trait ICallsBack<TContractState> {
            fn call_back(ref self: TContractState, address: ContractAddress);
        }

        #[test]
        fn test_calling_test_fails() {
            let contract = declare('CallsBack');
            let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher = ICallsBackDispatcher { contract_address: contract_address };
            dispatcher.call_back(test_address());
        }
    "#
        ),
        Contract::new(
            "CallsBack",
            indoc!(
                r#"
                #[starknet::contract]
                mod CallsBack {
                    use result::ResultTrait;
                    use starknet::ClassHash;
                    use starknet::{library_call_syscall, ContractAddress};

                    #[storage]
                    struct Storage {
                    }

                    #[starknet::interface]
                    trait IDontExist<TContractState> {
                        fn test_calling_test_fails(ref self: TContractState);
                    }
        

                    #[external(v0)]
                    fn call_back(ref self: ContractState, address: ContractAddress) {
                        let dispatcher = IDontExistDispatcher{contract_address: address};
                        dispatcher.test_calling_test_fails();
                    }
                }
                "#
            )
        )
    );

    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(result, "test_calling_test_fails", "Entry point");
    assert_case_output_contains!(result, "test_calling_test_fails", "not found in contract");
}

#[test]
fn test_storage_access_default_values() {
    let test = test_case!(indoc!(
        r#"
        #[starknet::contract]
        mod Contract {
            #[derive(starknet::Store, Drop)]
            struct CustomStruct {
                a: felt252,
                b: felt252,
            }
            #[storage]
            struct Storage {
                balance: felt252,
                legacy_map: LegacyMap<felt252, felt252>,
                custom_struct: CustomStruct,
            }
        }

        use tests::test_case::Contract::balanceContractMemberStateTrait;
        use tests::test_case::Contract::legacy_mapContractMemberStateTrait;
        use tests::test_case::Contract::custom_structContractMemberStateTrait;

        #[test]
        fn testing_storage_defaults() {
            let mut state = Contract::contract_state_for_testing();
            let default_felt252 = state.balance.read();
            assert(default_felt252 == 0, 'Incorrect storage value');

            let default_map_value = state.legacy_map.read(22);
            assert(default_map_value == 0, 'Incorrect map value');

            let default_custom_struct = state.custom_struct.read();
            assert(default_custom_struct.a == 0, 'Invalid cs.a value');
            assert(default_custom_struct.b == 0, 'Invalid cs.b value');
        }
    "#
    ),);

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn test_simple_cheatcodes() {
    let test = test_case!(indoc!(
        r#"
        use result::ResultTrait;
        use box::BoxTrait;
        use serde::Serde;
        use starknet::ContractAddress;
        use array::SpanTrait;
        use starknet::ContractAddressIntoFelt252;
        use snforge_std::{
            start_prank, stop_prank,
            start_roll, stop_roll,
            start_warp, stop_warp,
            start_spoof, stop_spoof,
            TxInfoMockTrait,
            test_address
        };

        #[test]
        fn test_prank_test_state() {
            let test_address: ContractAddress = test_address();
            let caller_addr_before = starknet::get_caller_address();
            let target_caller_address: ContractAddress = (123_felt252).try_into().unwrap();

            start_prank(test_address, target_caller_address);
            let caller_addr_after = starknet::get_caller_address();
            assert(caller_addr_after==target_caller_address, caller_addr_after.into());

            stop_prank(test_address);
            let caller_addr_after = starknet::get_caller_address();
            assert(caller_addr_after==caller_addr_before, caller_addr_before.into());
        }

        #[test]
        fn test_roll_test_state() {
            let test_address: ContractAddress = test_address();
            let old_block_number = starknet::get_block_info().unbox().block_number;

            start_roll(test_address, 234);
            let new_block_number = starknet::get_block_info().unbox().block_number;
            assert(new_block_number == 234, 'Wrong block number');

            stop_roll(test_address);
            let new_block_number = starknet::get_block_info().unbox().block_number;
            assert(new_block_number == old_block_number, 'Block num did not change back');
        }

        #[test]
        fn test_warp_test_state() {
            let test_address: ContractAddress = test_address();
            let old_block_timestamp = starknet::get_block_info().unbox().block_timestamp;

            start_warp(test_address, 123);
            let new_block_timestamp = starknet::get_block_info().unbox().block_timestamp;
            assert(new_block_timestamp == 123, 'Wrong block timestamp');

            stop_warp(test_address);
            let new_block_timestamp = starknet::get_block_info().unbox().block_timestamp;
            assert(new_block_timestamp == old_block_timestamp, 'Timestamp did not change back')
        }

        #[test]
        fn test_spoof_test_state() {
            let test_address: ContractAddress = test_address();
            let old_tx_info = starknet::get_tx_info().unbox();

            let mut tx_info_mock = TxInfoMockTrait::default();
            tx_info_mock.transaction_hash = Option::Some(421);

            start_spoof(test_address, tx_info_mock);
            let new_tx_info = starknet::get_tx_info().unbox();
            assert(new_tx_info.nonce == old_tx_info.nonce, 'Wrong nonce');
            assert(new_tx_info.transaction_hash == 421, 'Wrong transaction_hash');

            stop_spoof(test_address);
            let new_tx_info = starknet::get_tx_info().unbox();
            assert(new_tx_info.nonce == old_tx_info.nonce, 'Wrong nonce');
            assert(
                new_tx_info.transaction_hash == old_tx_info.transaction_hash,
                'Wrong transaction_hash'
            )
        }
    "#
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn test_spy_events_simple() {
    let test = test_case!(indoc!(
        r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::SyscallResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy, EventFetcher,
                event_name_hash, EventAssertions, Event, SpyOn, test_address };

            #[test]
            fn test_expect_events_simple() {
                let contract_address = test_address();
                let mut spy = spy_events(SpyOn::One(contract_address));
                assert(spy._id == 0, 'Id should be 0');

                starknet::emit_event_syscall(array![1234].span(), array![2345].span()).unwrap_syscall();

                spy.assert_emitted(@array![
                    (
                        contract_address,
                        Event { keys: array![1234], data: array![2345] }
                    )
                ]);

                assert(spy.events.len() == 0, 'There should be no events left');
            }
        "#
    ),);

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn test_spy_struct_events() {
    let test = test_case!(indoc!(
        r#"
            use array::ArrayTrait;
            use snforge_std::{ 
                declare, ContractClassTrait, spy_events, 
                EventSpy, EventFetcher, 
                EventAssertions, Event, SpyOn, test_address 
            };
                
           #[starknet::contract]
            mod Emitter {
                use result::ResultTrait;
                use starknet::ClassHash;
                
                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {
                    ThingEmitted: ThingEmitted
                }
                
                #[derive(Drop, starknet::Event)]
                struct ThingEmitted {
                    thing: felt252
                }
    
                #[storage]
                struct Storage {}

                #[external(v0)]
                fn emit_event(
                    ref self: ContractState,
                ) {
                    self.emit(Event::ThingEmitted(ThingEmitted { thing: 420 }));
                }
            }

            #[test]
            fn test_expect_event_struct() {
                let contract_address = test_address();
                let mut spy = spy_events(SpyOn::One(contract_address));
                
                let mut testing_state = Emitter::contract_state_for_testing();
                Emitter::emit_event(ref testing_state);
                
                spy.assert_emitted(
                    @array![
                        (
                            contract_address,
                            Emitter::Event::ThingEmitted(Emitter::ThingEmitted { thing: 420 })
                        )
                    ]
                )
            }
        "#
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn test_inconsistent_syscall_pointers() {
    let test = test_case!(indoc!(
        r#"
        use starknet::ContractAddress;
        use starknet::info::get_block_number;
        use snforge_std::start_mock_call;

        #[starknet::interface]
        trait IContract<TContractState> {
            fn get_value(self: @TContractState, arg: ContractAddress) -> u128;
        }

        #[test]
        fn test_deploy_error_handling() {
            // verifies if SyscallHandler.syscal_ptr is incremented correctly when calling a contract
            let address = 'address'.try_into().unwrap();
            start_mock_call(address, 'get_value', 55);
            let contract = IContractDispatcher { contract_address: address };
            let value = contract.get_value(address);
            let block_number = get_block_number();
        }
    "#
    ),);
    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn test_caller_address_in_called_contract() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use array::ArrayTrait;
        use option::OptionTrait;
        use traits::TryInto;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        use snforge_std::{ declare, ContractClassTrait, test_address };

        #[starknet::interface]
        trait IPrankChecker<TContractState> {
            fn get_caller_address(ref self: TContractState) -> felt252;
        }

        #[starknet::interface]
        trait IConstructorPrankChecker<TContractState> {
            fn get_stored_caller_address(ref self: TContractState) -> ContractAddress;
        }

        #[test]
        fn test_contract() {
            let prank_checker = declare('PrankChecker');
            let contract_address_prank_checker = prank_checker.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher_prank_checker = IPrankCheckerDispatcher { contract_address: contract_address_prank_checker };

            assert(dispatcher_prank_checker.get_caller_address() == test_address().into(), 'Incorrect caller address');


            let constructor_prank_checker = declare('ConstructorPrankChecker');
            let contract_address_constructor_prank_checker = constructor_prank_checker.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher_constructor_prank_checker = IConstructorPrankCheckerDispatcher { contract_address: contract_address_constructor_prank_checker };

            assert(dispatcher_constructor_prank_checker.get_stored_caller_address() == test_address(), 'Incorrect caller address');

        }
    "#
        ),
        Contract::from_code_path(
            "PrankChecker".to_string(),
            Path::new("tests/data/contracts/prank_checker.cairo"),
        )
        .unwrap(),
        Contract::new(
            "ConstructorPrankChecker",
            indoc!(
                r#"
            use starknet::ContractAddress;

            #[starknet::interface]
            trait IConstructorPrankChecker<TContractState> {
                fn get_stored_caller_address(ref self: TContractState) -> ContractAddress;
            }

            #[starknet::contract]
            mod ConstructorPrankChecker {
                use starknet::ContractAddress;

                #[storage]
                struct Storage {
                    caller_address: ContractAddress,
                }

                #[constructor]
                fn constructor(ref self: ContractState) {
                    let address = starknet::get_caller_address();
                    self.caller_address.write(address);
                }

                #[external(v0)]
                impl IConstructorPrankChecker of super::IConstructorPrankChecker<ContractState> {
                    fn get_stored_caller_address(ref self: ContractState) -> ContractAddress {
                        self.caller_address.read()
                    }
                }
            }
        "#
            )
        )
    );
    let result = run_test_case(&test);

    assert_passed!(result);
}
