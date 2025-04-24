# `Token`

```rust
pub enum Token {
    STRK,
    Custom {
        contract_address: ContractAddress,
        balances_variable_selector: Felt,
    },
}

pub struct CustomToken {
    pub contract_address: ContractAddress,
    pub balances_variable_selector: felt252,
}
```

`Token` is an enum used to specify ERC20 token for which the balance should be cheated. It can be either `STRK` or a custom token.
- `STRK` is the default STRK token (predeployed in every test case).
- `Custom` allows to specify a custom token by providing its contract address and balances variable selector.

```rust
pub impl TokenImpl of TokenTrait {
    fn contract_address(self: Token) -> ContractAddress {
        match self {
            Token::STRK => STRK_CONTRACT_ADDRESS.try_into().unwrap(),
            Token::Custom(CustomToken { contract_address, .. }) => contract_address,
        }
    }

    fn balances_variable_selector(self: Token) -> felt252 {
        match self {
            Token::STRK => selector!("ERC20_balances"),
            Token::Custom(CustomToken { balances_variable_selector,
            .., }) => balances_variable_selector,
        }
    }
}
```
