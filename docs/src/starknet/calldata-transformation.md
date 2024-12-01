# Calldata Transformation

For the examples below, we will consider a dedicated contract - `DataTransformerContract`, defined
in `data_transformer_contract` project namespace.

It's declared on Sepolia network with class hash `0x02a9b456118a86070a8c116c41b02e490f3dcc9db3cad945b4e9a7fd7cec9168`.

It has a few methods accepting different types and items defined in its namespace:

```rust
// data_transformer_contract/src/lib.cairo

pub struct SimpleStruct {
    a: felt252
}

pub struct NestedStructWithField {
    a: SimpleStruct,
    b: felt252
}

pub enum Enum {
    One: (),
    Two: u128,
    Three: NestedStructWithField
}

#[starknet::contract]
pub mod DataTransformerContract {
    /* ... */

    use super::*;

    fn tuple_fn(self: @ContractState, a: (felt252, u8, Enum)) { ... }

    fn nested_struct_fn(self: @ContractState, a: NestedStructWithField) { ... }

    fn complex_fn(
        self: @ContractState,
        arr: Array<Array<felt252>>,
        one: u8,
        two: i16,
        three: ByteArray,
        four: (felt252, u32),
        five: bool,
        six: u256
    ) {
        ...
    }
}
```

A default form of calldata passed to commands requiring it is a series of hex-encoded felts:

<!-- { "contract_name": "DataTransformerContract" } -->
```shell
$ sncast call \
    --url http://127.0.0.1:5055 \
    --contract-address 0xcd7bbe72e64e86a894de5c8c9afa0ba9f0434765c52df822f18f5c93cc395f \
    --function tuple_fn \
    --calldata 0x10 0x3 0x0 \
    --block-id latest
```

> 💡 **Info**
> Cast **doesn't verify serialized calldata against the ABI**.\
> Only expression transformation checks types and arities of functions called on chain.

## Using `--arguments`

Instead of serializing calldata yourself, `sncast` allows passing it in a far more handy, human-readable form - as a
list of comma-separated Cairo expressions wrapped in single quotes. This can be achieved by using the `--arguments`
flag.
Cast will perform serialization automatically, based on an ABI of the contract
we interact with, following
the [Starknet specification](https://docs.starknet.io/architecture-and-concepts/smart-contracts/serialization-of-cairo-types/).

### Basic example

We can write the same command as above, but with arguments:

<!-- { "contract_name": "DataTransformerContract" } -->
```shell
$ sncast call \
    --url http://127.0.0.1:5055 \
    --contract-address 0xcd7bbe72e64e86a894de5c8c9afa0ba9f0434765c52df822f18f5c93cc395f \
    --function tuple_fn \
    --arguments '(0x10, 3, hello_sncast::data_transformer_contract::Enum::One)' \
    --block-id latest
```

getting the same result.
Note that the arguments must be:

* provided as a single string
* comma (`,`) separated

> 📝 **Note**
> User-defined items such as enums and structs should be referred to depending on a way they are defined in ABI.\
> In general, paths to items have form: `<project-name>::<module-path>::<item-name>`.

## Supported Expressions

Cast supports most important Cairo corelib types:

* `bool`
* signed integers (`i8`, `i16`, `i32`, `i64`, `i128`)
* unsigned integers (`u8`, `u16`, `u32`, `u64`, `u96`, `u128`, `u256`, `u384`, `u512`)
* `felt252` (numeric literals and `'shortstrings'`)
* `ByteArray`
* `ContractAddress`
* `ClassHash`
* `StorageAddress`
* `EthAddress`
* `bytes31`
* `Array` - using `array![]` macro

Numeric types (primitives and `felt252`) can be paseed with type suffix specified for example `--arguments 420_u64`.

> 📝 **Note**
> Only **constant** expressions are supported. Defining and referencing variables and calling functions (either builtin,
> user-defined or external) is not allowed.

### More Complex Examples

1. `complex_fn` - different data types:

<!-- { "contract_name": "DataTransformerContract" } -->
```shell
$ sncast call \
    --url http://127.0.0.1:5055 \
    --contract-address 0xcd7bbe72e64e86a894de5c8c9afa0ba9f0434765c52df822f18f5c93cc395f \
    --function complex_fn \
    --arguments \
'array![array![1, 2], array![3, 4, 5], array![6]],'\
'12,'\
'-128_i8,'\
'"Some string (a ByteArray)",'\
"('a shortstring', 32_u32),"\
'true,'\
'0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff' \
    --block-id latest
```

> 📝 **Note**
> In bash and similar shells indentation and whitespace matters when providing multiline strings with `\`
>
> Remember  **not to indent** any line and **not to add whitespace before the `\` character**.

Alternatively, you can continue the single quote for multiple lines.

<!-- { "contract_name": "DataTransformerContract" } -->
```shell
$ sncast call \
    --url http://127.0.0.1:5055 \
    --contract-address 0xcd7bbe72e64e86a894de5c8c9afa0ba9f0434765c52df822f18f5c93cc395f \
    --function complex_fn \
    --arguments 'array![array![1, 2], array![3, 4, 5], array![6]],
12,
-128_i8,
"Some string (a ByteArray)",
('\''a shortstring'\'', 32_u32),
true,
0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff' \
    --block-id latest
```

> 📝 **Note**
> In bash and similar shells any `'` must be escaped correctly.
>
> This is also true for `"` when using `"` to wrap the arguments instead of `'`.

2. `nested_struct_fn` - struct nesting:

<!-- { "contract_name": "DataTransformerContract" } -->
```shell
$ sncast call \
    --url http://127.0.0.1:5055 \
    --contract-address 0xcd7bbe72e64e86a894de5c8c9afa0ba9f0434765c52df822f18f5c93cc395f \
    --function nested_struct_fn \
    --arguments \
'hello_sncast::data_transformer_contract::NestedStructWithField {'\
'    a: hello_sncast::data_transformer_contract::SimpleStruct { a: 10 },'\
'    b: 12'\
'}'\
      --block-id latest
```
