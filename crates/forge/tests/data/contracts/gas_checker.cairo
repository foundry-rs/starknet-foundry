#[starknet::interface]
trait IGasChecker<TContractState> {
    fn keccak(self: @TContractState);
    fn range_check(self: @TContractState);
    fn bitwise(self: @TContractState);
    fn pedersen(self: @TContractState);
    fn poseidon(self: @TContractState);
    fn ec_op(self: @TContractState);
}

#[starknet::contract]
mod GasChecker {
    use core::{ec, ec::{EcPoint, EcPointTrait}};

    #[storage]
    struct Storage {}

    #[external(v0)]
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

    }
}