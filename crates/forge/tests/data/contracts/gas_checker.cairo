#[starknet::interface]
trait IGasChecker<TContractState> {
    fn keccak(self: @TContractState);
    fn range_check(self: @TContractState);
    fn bitwise(self: @TContractState);
    fn pedersen(self: @TContractState);
    fn poseidon(self: @TContractState);
    fn ec_op(self: @TContractState);

    fn change_balance(ref self: TContractState, new_balance: u64);
    fn send_l1_message(self: @TContractState);
}

#[starknet::contract]
mod GasChecker {
    use core::{ec, ec::{EcPoint, EcPointTrait}};

    #[storage]
    struct Storage {
        balance: u64
    }

    #[abi(embed_v0)]
    impl IGasCheckerImpl of super::IGasChecker<ContractState> {
        fn keccak(self: @ContractState) {
            keccak::keccak_u256s_le_inputs(array![1].span());
        }

        fn range_check(self: @ContractState) {
            assert(1_u8 >= 1_u8, 'error message');
            assert(1_u8 >= 1_u8, 'error message');
            assert(1_u8 >= 1_u8, 'error message');
            assert(1_u8 >= 1_u8, 'error message');
            assert(1_u8 >= 1_u8, 'error message');
            assert(1_u8 >= 1_u8, 'error message');
            assert(1_u8 >= 1_u8, 'error message');
            assert(1_u8 >= 1_u8, 'error message');
            assert(1_u8 >= 1_u8, 'error message');
        }

        fn bitwise(self: @ContractState) {
            let bitwise = 1_u8 & 1_u8;
            let bitwise = 1_u8 & 1_u8;
            let bitwise = 1_u8 & 1_u8;
            let bitwise = 1_u8 & 1_u8;
            let bitwise = 1_u8 & 1_u8;
            let bitwise = 1_u8 & 1_u8;
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
        }

        fn ec_op(self: @ContractState) {
            EcPointTrait::new_from_x(1).unwrap().mul(2);
        }

        fn change_balance(ref self: ContractState, new_balance: u64) {
            self.balance.write(new_balance);
        }

        fn send_l1_message(self: @ContractState) {
            starknet::send_message_to_l1_syscall(1, array![1].span()).unwrap();
        }
    }
}
