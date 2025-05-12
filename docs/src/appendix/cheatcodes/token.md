# `Token`

```rust
pub enum Token {
    STRK,
}
```

`Token` is an enum used to specify ERC20 token for which the balance should be cheated. List of available tokens:
- `STRK` is the default STRK token (predeployed in every test case).

```rust
pub impl TokenImpl of TokenTrait {
    fn contract_address(self: Token) -> ContractAddress {
        match self {
            Token::STRK => STRK_CONTRACT_ADDRESS.try_into().unwrap(),
        }
    }

    fn balances_variable_selector(self: Token) -> felt252 {
        match self {
            Token::STRK => selector!("ERC20_balances"),
        }
    }
}
```
