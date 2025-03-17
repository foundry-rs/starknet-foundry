use starknet::ContractAddress;

#[derive(Drop, Serde, Copy)]
pub struct TransferRequest {
    pub recipient: ContractAddress,
    pub amount: u256,
}

/// Interface representing `TokenSender` contract functionality.
#[starknet::interface]
pub trait ITokenSender<TContractState> {
    /// Function to send tokens to multiple recipients in a single transaction.
    /// - `token_address` - The address of the token contract
    /// - `transfer_list` - The list of transfers to perform
    fn multisend(
        ref self: TContractState,
        token_address: ContractAddress,
        transfer_list: Array<TransferRequest>,
    ) -> ();
}

#[starknet::contract]
pub mod TokenSender {
    use openzeppelin_token::erc20::{ERC20ABIDispatcher, ERC20ABIDispatcherTrait};
    use starknet::{ContractAddress, get_caller_address, get_contract_address};
    use super::TransferRequest;


    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        TransferSent: TransferSent,
    }

    #[derive(Drop, starknet::Event)]
    pub struct TransferSent {
        #[key]
        pub recipient: ContractAddress,
        pub token_address: ContractAddress,
        pub amount: u256,
    }


    #[constructor]
    fn constructor(ref self: ContractState) {}

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl TokenSender of super::ITokenSender<ContractState> {
        fn multisend(
            ref self: ContractState,
            token_address: ContractAddress,
            transfer_list: Array<TransferRequest>,
        ) {
            // Create an ERC20 dispatcher to interact with the given token contract.
            let erc20 = ERC20ABIDispatcher { contract_address: token_address };

            // Compute total amount to be transferred.
            let mut total_amount: u256 = 0;
            for t in transfer_list.span() {
                total_amount += *t.amount;
            };

            // Transfer the total amount from the caller to this contract.
            erc20.transfer_from(get_caller_address(), get_contract_address(), total_amount);

            // Distribute tokens to each recipient in the transfer list.
            for t in transfer_list.span() {
                erc20.transfer(*t.recipient, *t.amount);
                self
                    .emit(
                        TransferSent {
                            recipient: *t.recipient,
                            token_address: token_address,
                            amount: *t.amount,
                        },
                    );
            };
        }
    }
}
