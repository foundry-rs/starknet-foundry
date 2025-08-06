const MINTER_ROLE: felt252 = selector!("MINTER_ROLE");

#[starknet::interface]
pub trait IMyContract<TContractState> {
    fn mint(ref self: TContractState);
}

#[starknet::contract]
mod MyContract {
    use component_macros::oz_ac_component::{AccessControlComponent, SRC5Component};
    use starknet::ContractAddress;
    use super::MINTER_ROLE;

    component!(path: AccessControlComponent, storage: accesscontrol, event: AccessControlEvent);
    component!(path: SRC5Component, storage: src5, event: SRC5Event);


    // AccessControl
    #[abi(embed_v0)]
    impl AccessControlImpl =
        AccessControlComponent::AccessControlImpl<ContractState>;
    impl AccessControlInternalImpl = AccessControlComponent::InternalImpl<ContractState>;

    // SRC5
    #[abi(embed_v0)]
    impl SRC5Impl = SRC5Component::SRC5Impl<ContractState>;

    #[storage]
    struct Storage {
        #[substorage(v0)]
        accesscontrol: AccessControlComponent::Storage,
        #[substorage(v0)]
        src5: SRC5Component::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        #[flat]
        AccessControlEvent: AccessControlComponent::Event,
        #[flat]
        SRC5Event: SRC5Component::Event,
    }

    #[constructor]
    fn constructor(ref self: ContractState, minter: ContractAddress) {
        // AccessControl-related initialization
        self.accesscontrol.initializer();
        self.accesscontrol._grant_role(MINTER_ROLE, minter);
    }

    #[abi(embed_v0)]
    impl MyContractImpl of super::IMyContract<ContractState> {
        /// This function can only be called by a minter.
        fn mint(ref self: ContractState) {
            self.accesscontrol.assert_only_role(MINTER_ROLE);
        }
    }
}
