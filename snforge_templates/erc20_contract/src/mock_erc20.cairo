/// Example ERC20 token contract created with openzeppelin dependency.
/// Full guide and documentation can be found at:
/// https://docs.openzeppelin.com/contracts-cairo/1.0.0/guides/erc20-supply
#[starknet::contract]
pub mod MockERC20 {
    use openzeppelin_token::erc20::{ERC20Component, ERC20HooksEmptyImpl};
    use starknet::ContractAddress;

    /// Declare the ERC20 component for this contract.
    /// This allows the contract to inherit ERC20 functionalities.
    component!(path: ERC20Component, storage: erc20, event: ERC20Event);

    /// Define ERC20 public interface.
    #[abi(embed_v0)]
    impl ERC20MixinImpl = ERC20Component::ERC20MixinImpl<ContractState>;

    /// Define internal implementation, allowing internal modifications like minting.
    impl ERC20InternalImpl = ERC20Component::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        #[substorage(v0)]
        erc20: ERC20Component::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        #[flat]
        ERC20Event: ERC20Component::Event,
    }

    #[constructor]
    fn constructor(ref self: ContractState, initial_supply: u256, recipient: ContractAddress) {
        let name = "MockToken";
        let symbol = "MTK";

        /// Initialize the contract by setting the token name and symbol.
        self.erc20.initializer(name, symbol);
        /// Create `initial_supply` amount of tokens and assigns them to `recipient`.
        self.erc20.mint(recipient, initial_supply);
    }
}
