# `Token`

```rust
pub enum Token {
    STRK,
    ETH,
    Custom: CustomToken,
}

pub struct CustomToken {
    pub contract_address: ContractAddress,
    pub balances_variable_selector: felt252,
}
```

`Token` is an enum used to specify ERC20 token for which the balance should be cheated. Possible variants are:
- `STRK` is the default STRK token (predeployed in every test case).
- `ETH` is the default ETH token (predeployed in every test case).
- `Custom` allows to specify a custom token by providing its contract address and balances variable selector.

```rust
pub impl TokenImpl of TokenTrait {
    fn contract_address(self: Token) -> ContractAddress {
        match self {
            Token::STRK => STRK_CONTRACT_ADDRESS.try_into().unwrap(),
            Token::ETH => ETH_CONTRACT_ADDRESS.try_into().unwrap(),
            Token::Custom(CustomToken { contract_address, .. }) => contract_address,
        }
    }

    fn balances_variable_selector(self: Token) -> felt252 {
        match self {
            Token::STRK | TOKEN::ETH => selector!("ERC20_balances"),
            Token::Custom(CustomToken { balances_variable_selector,
            .., }) => balances_variable_selector,
        }
    }
}
```

## Balances variable selector

`balances_variable_selector` is simply a selector of the storage variable, which holds the mapping of balances -> amounts. The name of variable isn't specified by ERC20 standard (it can have any name), hence we allow to specify it. Let's have a part of example ERC20 contract storage:

```rust
    ...
    #[storage]
    struct Storage {
        ...
        balances: Map<ContractAddress, u256>,
        ...
    }
    ...
```

In the above example, `balances_variable_selector` would have following value:

```rust
let token = Token::Custom(
        CustomToken {
            contract_address: ...,
            balances_variable_selector: selector!("balances"),
        },
    );
```