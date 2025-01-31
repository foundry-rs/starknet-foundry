use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_case_output_contains, assert_failed, assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn storage_access_from_tests() {
    let test = test_case!(indoc!(
        r"
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


        #[test]
        fn storage_access_from_tests() {
            let mut state = Contract::contract_state_for_testing();
            state.balance.write(10);
            
            let value = Contract::InternalImpl::internal_function(@state);
            assert(value == 10, 'Incorrect storage value');
        }
    "
    ),);

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn simple_syscalls() {
    let test = test_case!(
        indoc!(
            r#"
        use starknet::info::{get_execution_info, TxInfo};
        use result::ResultTrait;
        use box::BoxTrait;
        use serde::Serde;
        use starknet::{ContractAddress, get_block_hash_syscall};
        use array::SpanTrait;
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, test_address };

        #[starknet::interface]
        trait ICheatTxInfoChecker<TContractState> {
            fn get_tx_hash(ref self: TContractState) -> felt252;
            fn get_nonce(ref self: TContractState) -> felt252;
            fn get_account_contract_address(ref self: TContractState) -> ContractAddress;
            fn get_signature(ref self: TContractState) -> Span<felt252>;
            fn get_version(ref self: TContractState) -> felt252;
            fn get_max_fee(ref self: TContractState) -> u128;
            fn get_chain_id(ref self: TContractState) -> felt252;
        }
        #[starknet::interface]
        trait ICheatBlockNumberChecker<TContractState> {
            fn get_block_number(ref self: TContractState) -> u64;
        }

        #[starknet::interface]
        trait ICheatBlockTimestampChecker<TContractState> {
            fn get_block_timestamp(ref self: TContractState) -> u64;
        }

        #[starknet::interface]
        trait ICheatSequencerAddressChecker<TContractState> {
            fn get_sequencer_address(ref self: TContractState) -> ContractAddress;
        }

        #[test]
        fn simple_syscalls() {
            let exec_info = get_execution_info().unbox();
            assert(exec_info.caller_address.into() == 0, 'Incorrect caller address');
            assert(exec_info.contract_address == test_address(), exec_info.contract_address.into());
            // Hash of TEST_CASE_SELECTOR
            assert(exec_info.entry_point_selector.into() == 655947323460646800722791151288222075903983590237721746322261907338444055163, 'Incorrect entry point selector');

            let block_info = exec_info.block_info.unbox();

            let contract_cheat_block_number = declare("CheatBlockNumberChecker").unwrap().contract_class();
            let (contract_address_cheat_block_number, _) = contract_cheat_block_number.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher_cheat_block_number = ICheatBlockNumberCheckerDispatcher { contract_address: contract_address_cheat_block_number };

            let contract_cheat_block_timestamp = declare("CheatBlockTimestampChecker").unwrap().contract_class();
            let (contract_address_cheat_block_timestamp, _) = contract_cheat_block_timestamp.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher_cheat_block_timestamp = ICheatBlockTimestampCheckerDispatcher { contract_address: contract_address_cheat_block_timestamp };

            let contract_cheat_sequencer_address = declare("CheatSequencerAddressChecker").unwrap().contract_class();
            let (contract_address_cheat_sequencer_address, _) = contract_cheat_sequencer_address.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher_cheat_sequencer_address = ICheatSequencerAddressCheckerDispatcher { contract_address: contract_address_cheat_sequencer_address };

            assert(dispatcher_cheat_block_number.get_block_number() == block_info.block_number, 'Invalid block number');
            assert(dispatcher_cheat_block_timestamp.get_block_timestamp() == block_info.block_timestamp, 'Invalid block timestamp');
            assert(dispatcher_cheat_sequencer_address.get_sequencer_address() == block_info.sequencer_address, 'Invalid sequencer address');

            let contract = declare("CheatTxInfoChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher = ICheatTxInfoCheckerDispatcher { contract_address };

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
            "CheatTxInfoChecker".to_string(),
            Path::new("tests/data/contracts/cheat_tx_info_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "CheatBlockNumberChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_number_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "CheatBlockTimestampChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_timestamp_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "CheatSequencerAddressChecker".to_string(),
            Path::new("tests/data/contracts/cheat_sequencer_address_checker.cairo")
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn get_block_hash_syscall_in_dispatcher() {
    let test = test_case!(
        indoc!(
            r#"
        use starknet::info::{get_execution_info, TxInfo};
        use result::ResultTrait;
        use box::BoxTrait;
        use serde::Serde;
        use starknet::{ContractAddress, get_block_hash_syscall};
        use array::SpanTrait;
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, test_address };

        #[starknet::interface]
        trait BlockHashChecker<TContractState> {
            fn write_block(ref self: TContractState);
            fn read_block_hash(self: @TContractState) -> felt252;
        }

        #[test]
        fn get_block_hash_syscall_in_dispatcher() {
            let block_hash_checker = declare("BlockHashChecker").unwrap().contract_class();
            let (block_hash_checker_address, _) = block_hash_checker.deploy(@ArrayTrait::new()).unwrap();
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

    assert_passed(&result);
}

#[test]
fn library_calls() {
    let test = test_case!(
        indoc!(
            r#"
        use core::clone::Clone;
        use result::ResultTrait;
        use starknet::{ ClassHash, library_call_syscall, ContractAddress };
        use snforge_std::{ declare, DeclareResultTrait };

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
        fn library_calls() {
            let class_hash = declare("LibraryContract").unwrap().contract_class().class_hash.clone();
            let lib_dispatcher = ILibraryContractLibraryDispatcher { class_hash };
            let value = lib_dispatcher.get_value();
            assert(value == 0, 'Incorrect state');
            lib_dispatcher.set_value(10);
            let value = lib_dispatcher.get_value();
            assert(value == 10, 'Incorrect state');
        }
    "#
        ),
        Contract::new(
            "LibraryContract",
            indoc!(
                r"
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

                #[starknet::contract]
                mod LibraryContract {
                    use result::ResultTrait;
                    use starknet::ClassHash;
                    use starknet::library_call_syscall;

                    #[storage]
                    struct Storage {
                        value: felt252
                    }

                    #[abi(embed_v0)]
                    impl LibraryContractImpl of super::ILibraryContract<ContractState> {
                        fn get_value(
                            self: @ContractState,
                        ) -> felt252 {
                           self.value.read()
                        }

                        fn set_value(
                            ref self: ContractState,
                            number: felt252
                        ) {
                           self.value.write(number);
                        }
                    }
                }
                "
            )
        )
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn disabled_syscalls() {
    let test = test_case!(
        indoc!(
            r"
        use result::ResultTrait;
        use starknet::{ClassHash, deploy_syscall, replace_class_syscall, get_block_hash_syscall};
        use snforge_std::declare;
        
        #[test]
        fn disabled_syscalls() {
            let value : ClassHash = 'xd'.try_into().unwrap();
            replace_class_syscall(value).unwrap();
        }
    "
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "disabled_syscalls",
        "Replace class can't be used in tests",
    );
}

#[test]
fn get_block_hash() {
    let test = test_case!(indoc!(
        r"
        use result::ResultTrait;
        use box::BoxTrait;
        use starknet::{get_block_hash_syscall, get_block_info};

        #[test]
        fn get_block_hash() {
            let block_info = get_block_info().unbox();
            let hash = get_block_hash_syscall(block_info.block_number - 10).unwrap();
            assert(hash == 0, 'Hash not zero');
        }
    "
    ));

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn cant_call_test_contract() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use starknet::{ClassHash, ContractAddress, deploy_syscall, replace_class_syscall, get_block_hash_syscall};
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, test_address };

        #[starknet::interface]
        trait ICallsBack<TContractState> {
            fn call_back(ref self: TContractState, address: ContractAddress);
        }

        #[test]
        fn cant_call_test_contract() {
            let contract = declare("CallsBack").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher = ICallsBackDispatcher { contract_address: contract_address };
            dispatcher.call_back(test_address());
        }
    "#
        ),
        Contract::new(
            "CallsBack",
            indoc!(
                r"
                use starknet::ContractAddress;

                #[starknet::interface]
                trait ICallsBack<TContractState> {
                    fn call_back(ref self: TContractState, address: ContractAddress);
                }

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
        

                    #[abi(embed_v0)]
                    impl CallsBackImpl of super::ICallsBack<ContractState> {
                        fn call_back(ref self: ContractState, address: ContractAddress) {
                            let dispatcher = IDontExistDispatcher{contract_address: address};
                            dispatcher.test_calling_test_fails();
                        }
                    }
                }
                "
            )
        )
    );

    let result = run_test_case(&test);

    assert_failed(&result);
    assert_case_output_contains(&result, "cant_call_test_contract", "ENTRYPOINT_NOT_FOUND");
    assert_case_output_contains(&result, "cant_call_test_contract", "ENTRYPOINT_FAILED");
}

#[test]
fn storage_access_default_values() {
    let test = test_case!(indoc!(
        r"
        #[starknet::contract]
        mod Contract {
         use starknet::storage::{
            StoragePointerWriteAccess, StorageMapReadAccess, StoragePathEntry, Map };
            #[derive(starknet::Store, Drop)]
            struct CustomStruct {
                a: felt252,
                b: felt252,
            }
            #[storage]
            struct Storage {
                balance: felt252,
                legacy_map: Map<felt252, felt252>,
                custom_struct: CustomStruct,
            }
        }


        #[test]
        fn storage_access_default_values() {
            let mut state = Contract::contract_state_for_testing();
            let default_felt252 = state.balance.read();
            assert(default_felt252 == 0, 'Incorrect storage value');

            let default_map_value = state.legacy_map.read(22);
            assert(default_map_value == 0, 'Incorrect map value');

            let default_custom_struct = state.custom_struct.read();
            assert(default_custom_struct.a == 0, 'Invalid cs.a value');
            assert(default_custom_struct.b == 0, 'Invalid cs.b value');
        }
    "
    ),);

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
#[allow(clippy::too_many_lines)]
fn simple_cheatcodes() {
    let test = test_case!(indoc!(
        r"
        use result::ResultTrait;
        use box::BoxTrait;
        use serde::Serde;
        use starknet::ContractAddress;
        use array::SpanTrait;
        use starknet::ContractAddressIntoFelt252;
        use snforge_std::{
            start_cheat_sequencer_address, stop_cheat_sequencer_address,
            start_cheat_caller_address, stop_cheat_caller_address,
            start_cheat_block_number, stop_cheat_block_number,
            start_cheat_block_timestamp, stop_cheat_block_timestamp,
            start_cheat_transaction_hash, stop_cheat_transaction_hash,
            test_address, CheatSpan
        };
        use starknet::{
            SyscallResultTrait, SyscallResult, syscalls::get_execution_info_v2_syscall,
        };

        #[test]
        fn cheat_caller_address_test_state() {
            let test_address: ContractAddress = test_address();
            let caller_addr_before = starknet::get_caller_address();
            let target_caller_address: ContractAddress = (123_felt252).try_into().unwrap();

            start_cheat_caller_address(test_address, target_caller_address);
            let caller_addr_after = starknet::get_caller_address();
            assert(caller_addr_after==target_caller_address, caller_addr_after.into());

            stop_cheat_caller_address(test_address);
            let caller_addr_after = starknet::get_caller_address();
            assert(caller_addr_after==caller_addr_before, caller_addr_before.into());
        }

        #[test]
        fn cheat_block_number_test_state() {
            let test_address: ContractAddress = test_address();
            let old_block_number = starknet::get_block_info().unbox().block_number;

            start_cheat_block_number(test_address, 234);
            let new_block_number = starknet::get_block_info().unbox().block_number;
            assert(new_block_number == 234, 'Wrong block number');

            stop_cheat_block_number(test_address);
            let new_block_number = starknet::get_block_info().unbox().block_number;
            assert(new_block_number == old_block_number, 'Block num did not change back');
        }

        #[test]
        fn cheat_block_timestamp_test_state() {
            let test_address: ContractAddress = test_address();
            let old_block_timestamp = starknet::get_block_info().unbox().block_timestamp;

            start_cheat_block_timestamp(test_address, 123);
            let new_block_timestamp = starknet::get_block_info().unbox().block_timestamp;
            assert(new_block_timestamp == 123, 'Wrong block timestamp');

            stop_cheat_block_timestamp(test_address);
            let new_block_timestamp = starknet::get_block_info().unbox().block_timestamp;
            assert(new_block_timestamp == old_block_timestamp, 'Timestamp did not change back')
        }

        #[test]
        fn cheat_sequencer_address_test_state() {
            let test_address: ContractAddress = test_address();
            let old_sequencer_address = starknet::get_block_info().unbox().sequencer_address;

            start_cheat_sequencer_address(test_address, 123.try_into().unwrap());
            let new_sequencer_address = starknet::get_block_info().unbox().sequencer_address;
            assert(new_sequencer_address == 123.try_into().unwrap(), 'Wrong sequencer address');

            stop_cheat_sequencer_address(test_address);
            let new_sequencer_address = starknet::get_block_info().unbox().sequencer_address;
            assert(new_sequencer_address == old_sequencer_address, 'Sequencer addr did not revert')
        }

        #[test]
        fn transaction_hash_test_state() {
            let test_address: ContractAddress = test_address();
            let old_tx_info = starknet::get_tx_info().unbox();
            let old_tx_info_v2 = get_tx_info_v2().unbox();

            start_cheat_transaction_hash(test_address, 421);

            let new_tx_info = starknet::get_tx_info().unbox();
            let new_tx_info_v2 = get_tx_info_v2().unbox();

            assert(new_tx_info.nonce == old_tx_info.nonce, 'Wrong nonce');
            assert(new_tx_info_v2.tip == old_tx_info_v2.tip, 'Wrong tip');
            assert(new_tx_info.transaction_hash == 421, 'Wrong transaction_hash');

            stop_cheat_transaction_hash(test_address);
            
            let new_tx_info = starknet::get_tx_info().unbox();
            let new_tx_info_v2 = get_tx_info_v2().unbox();

            assert(new_tx_info.nonce == old_tx_info.nonce, 'Wrong nonce');
            assert(new_tx_info_v2.tip == old_tx_info_v2.tip, 'Wrong tip');
            assert(
                new_tx_info.transaction_hash == old_tx_info.transaction_hash,
                'Wrong transaction_hash'
            )
        }

        fn get_execution_info_v2() -> Box<starknet::info::v2::ExecutionInfo> {
            get_execution_info_v2_syscall().unwrap_syscall()
        }

        fn get_tx_info_v2() -> Box<starknet::info::v2::TxInfo> {
            get_execution_info_v2().unbox().tx_info
        }
    "
    ));

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn spy_events_simple() {
    let test = test_case!(indoc!(
        r"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::SyscallResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, spy_events, Event, EventSpy,
                EventSpyTrait, EventSpyAssertionsTrait, EventsFilterTrait, test_address
            };

            #[test]
            fn spy_events_simple() {
                let contract_address = test_address();
                let mut spy = spy_events();
                // assert(spy._event_offset == 0, 'Events offset should be 0'); TODO(#2765)

                starknet::emit_event_syscall(array![1234].span(), array![2345].span()).unwrap_syscall();

                spy.assert_emitted(@array![
                    (
                        contract_address,
                        Event { keys: array![1234], data: array![2345] }
                    )
                ]);
            }
        "
    ),);

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn spy_struct_events() {
    let test = test_case!(indoc!(
        r"
            use array::ArrayTrait;
            use snforge_std::{
                declare, ContractClassTrait, spy_events, EventSpy,
                EventSpyTrait, EventSpyAssertionsTrait, EventsFilterTrait, test_address
            };

            #[starknet::interface]
            trait IEmitter<TContractState> {
              fn emit_event(ref self: TContractState);
            }
                
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

                #[abi(embed_v0)]
                impl EmitterImpl of super::IEmitter<ContractState> {
                    fn emit_event(
                        ref self: ContractState,
                    ) {
                        self.emit(Event::ThingEmitted(ThingEmitted { thing: 420 }));
                    }
                }
            }

            #[test]
            fn spy_struct_events() {
                let contract_address = test_address();
                let mut spy = spy_events();
                
                let mut testing_state = Emitter::contract_state_for_testing();
                Emitter::EmitterImpl::emit_event(ref testing_state);

                spy.assert_emitted(
                    @array![
                        (
                            contract_address,
                            Emitter::Event::ThingEmitted(Emitter::ThingEmitted { thing: 420 })
                        )
                    ]
                )
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn inconsistent_syscall_pointers() {
    let test = test_case!(indoc!(
        r#"
        use starknet::ContractAddress;
        use starknet::info::get_block_number;
        use snforge_std::{start_mock_call, MockCallData};

        #[starknet::interface]
        trait IContract<TContractState> {
            fn get_value(self: @TContractState, arg: ContractAddress) -> u128;
        }

        #[test]
        fn inconsistent_syscall_pointers() {
            // verifies if SyscallHandler.syscal_ptr is incremented correctly when calling a contract
            let address = 'address'.try_into().unwrap();
            start_mock_call(address, selector!("get_value"), MockCallData::Any, 55);
            let contract = IContractDispatcher { contract_address: address };
            contract.get_value(address);
            get_block_number();
        }
    "#
    ),);
    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn caller_address_in_called_contract() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use array::ArrayTrait;
        use option::OptionTrait;
        use traits::TryInto;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, test_address };

        #[starknet::interface]
        trait ICheatCallerAddressChecker<TContractState> {
            fn get_caller_address(ref self: TContractState) -> felt252;
        }

        #[starknet::interface]
        trait IConstructorCheatCallerAddressChecker<TContractState> {
            fn get_stored_caller_address(ref self: TContractState) -> ContractAddress;
        }

        #[test]
        fn caller_address_in_called_contract() {
            let cheat_caller_address_checker = declare("CheatCallerAddressChecker").unwrap().contract_class();
            let (contract_address_cheat_caller_address_checker, _) = cheat_caller_address_checker.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher_cheat_caller_address_checker = ICheatCallerAddressCheckerDispatcher { contract_address: contract_address_cheat_caller_address_checker };

            assert(dispatcher_cheat_caller_address_checker.get_caller_address() == test_address().into(), 'Incorrect caller address');


            let constructor_cheat_caller_address_checker = declare("ConstructorCheatCallerAddressChecker").unwrap().contract_class();
            let (contract_address_constructor_cheat_caller_address_checker, _) = constructor_cheat_caller_address_checker.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher_constructor_cheat_caller_address_checker = IConstructorCheatCallerAddressCheckerDispatcher { contract_address: contract_address_constructor_cheat_caller_address_checker };

            assert(dispatcher_constructor_cheat_caller_address_checker.get_stored_caller_address() == test_address(), 'Incorrect caller address');

        }
    "#
        ),
        Contract::from_code_path(
            "CheatCallerAddressChecker".to_string(),
            Path::new("tests/data/contracts/cheat_caller_address_checker.cairo"),
        )
        .unwrap(),
        Contract::new(
            "ConstructorCheatCallerAddressChecker",
            indoc!(
                r"
            use starknet::ContractAddress;

            #[starknet::interface]
            trait IConstructorCheatCallerAddressChecker<TContractState> {
                fn get_stored_caller_address(ref self: TContractState) -> ContractAddress;
            }

            #[starknet::contract]
            mod ConstructorCheatCallerAddressChecker {
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

                #[abi(embed_v0)]
                impl IConstructorCheatCallerAddressChecker of super::IConstructorCheatCallerAddressChecker<ContractState> {
                    fn get_stored_caller_address(ref self: ContractState) -> ContractAddress {
                        self.caller_address.read()
                    }
                }
            }
        "
            )
        )
    );
    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn felt252_dict_usage() {
    let test = test_case!(indoc!(
        r"
        #[starknet::contract]
        mod DictUsingContract {
            use core::num::traits::{One};
            
            fn unique_count(mut ary: Array<felt252>) -> u32 {
                let mut dict: Felt252Dict<felt252> = Default::default();
                let mut counter = 0;
                // TODO
                loop {
                    match ary.pop_front() {
                        Option::Some(value) => {
                            if dict.get(value).is_one() {
                                continue;
                            }
                            dict.insert(value, One::one());
                            counter += 1;
                        },
                        Option::None => { break; }
                    }
                };
                counter
            }

            #[storage]
            struct Storage {
                unique_count: u32
            }
        
            #[constructor]
            fn constructor(ref self: ContractState, values: Array<felt252>) {
                self.unique_count.write(unique_count(values));
            }
        
            #[external(v0)]
            fn get_unique(self: @ContractState) -> u32 {
                self.unique_count.read()
            }
            #[external(v0)]
            fn write_unique(ref self: ContractState, values: Array<felt252>) {
                self.unique_count.write(unique_count(values));
            }
        }
        
        #[test]
        fn test_dict_in_constructor() {
            let mut testing_state = DictUsingContract::contract_state_for_testing();
            DictUsingContract::constructor(
                ref testing_state, 
                array![1, 2, 3, 3, 3, 3 ,3, 4, 4, 4, 4, 4, 5, 5, 5, 5]
            );
            
            assert(DictUsingContract::get_unique(@testing_state) == 5_u32, 'wrong unq ctor');
            
            DictUsingContract::write_unique(
                ref testing_state, 
                array![1, 2, 3, 3, 3, 3 ,3, 4, 4, 4, 4, 4]
            );
            
            assert(DictUsingContract::get_unique(@testing_state) == 4_u32, ' wrote wrong unq');
        }
        "
    ));

    let result = run_test_case(&test);

    assert_passed(&result);
}
