#[starknet::interface]
trait IGasChecker<TContractState> {
    fn keccak(self: @TContractState, repetitions: u32);
    fn range_check(self: @TContractState);
    fn bitwise(self: @TContractState, repetitions: u32);
    fn pedersen(self: @TContractState);
    fn poseidon(self: @TContractState);
    fn ec_op(self: @TContractState, repetitions: u32);

    fn change_balance(ref self: TContractState, new_balance: u64);
    fn send_l1_message(self: @TContractState);
    fn emit_event(self: @TContractState, n_keys_and_vals: u32);
}

#[starknet::contract]
mod GasChecker {
    use core::{ec, ec::{EcPoint, EcPointTrait}};
    use core::starknet::ContractAddress;

    #[storage]
    struct Storage {
        balance: u64,
    }

    #[abi(embed_v0)]
    impl IGasCheckerImpl of super::IGasChecker<ContractState> {
        fn keccak(self: @ContractState, repetitions: u32) {
            let mut i: u32 = 0;
            while i < repetitions {
                keccak::keccak_u256s_le_inputs(array![1].span());
                i += 1;
            }
        }

        fn range_check(self: @ContractState) {
            // Felt into ContractAddress conversion uses RangeCheck as implicit argument
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
            let x: ContractAddress = 1234.try_into().unwrap();
        }

        fn bitwise(self: @ContractState, repetitions: u32) {
            let mut i: u32 = 0;
            while i < repetitions {
                1_u8 & 1_u8;
                2_u8 & 2_u8;
                i += 1;
            }
        }

        fn pedersen(self: @ContractState) {
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
        }

        fn poseidon(self: @ContractState) {
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
            core::poseidon::hades_permutation(0, 0, 0);
        }

        fn ec_op(self: @ContractState, repetitions: u32) {
            let mut i: u32 = 0;
            while i < repetitions {
                EcPointTrait::new_from_x(1).unwrap().mul(2);
                i += 1;
            }
        }

        fn change_balance(ref self: ContractState, new_balance: u64) {
            self.balance.write(new_balance);
        }

        fn send_l1_message(self: @ContractState) {
            starknet::send_message_to_l1_syscall(1, array![1, 2, 3].span()).unwrap();
        }

        fn emit_event(self: @ContractState, n_keys_and_vals: u32) {
            let mut keys = array![];
            let mut values = array![];

            let mut i: u32 = 0;
            while i < n_keys_and_vals {
                keys.append('key');
                values.append(1);
                i += 1;
            };

            starknet::emit_event_syscall(keys.span(), values.span()).unwrap();
        }
    }

    #[l1_handler]
    fn handle_l1_message(ref self: ContractState, from_address: felt252) {
        keccak::keccak_u256s_le_inputs(array![1].span());
        keccak::keccak_u256s_le_inputs(array![1].span());
        keccak::keccak_u256s_le_inputs(array![1].span());
        keccak::keccak_u256s_le_inputs(array![1].span());
    }
}
