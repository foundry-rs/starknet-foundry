# Calldata Transformation

For the examples below, we will consider a dedicated contract - `DataTransformerContract`, defined in `data_transformer_contract` project namespace.

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

```shell
$ sncast --account myuser \
    call \
    --url http://127.0.0.1:5050 \
    --contract-address 0x016ad425af4585102e139d4fb2c76ce786d1aaa1cfcd88a51f3ed66601b23cdd \
    --function tuple_fn \
    --calldata 0x10 0x3 0x0 \
    --block-id latest
```

> 💡 **Info**
> Cast **doesn't verify serialized calldata against the ABI**.\
> Only expression transformation checks types and arities of functions called on chain.

However, `sncast` allows passing the data in far more handy, human-readable form - as a tuple of Cairo expressions.
When calldata is delivered in such form, Cast will perform serialization automatically, based on an ABI of the contract we interact with.

> 📝 **Note**
> Calldata transformation is enabled only when following requirements are met:
>
> * **Arguments form a tuple** (on a contrary to a standard serialized input which is whitespace-separated). Note the lack of commas in the example above - it signals Cast that the input is serialized
>
> * If there is only one single argument, **it is also written as a tuple**\
> (e.g. `--calldata (0x2137,)`)
>
> * Expressions should match the **Cairo syntax** and use **allowed items** - see [Supported expressions](./calldata-transformation.md#supported-expressions)

### Basic example

We can write the same command as above, but with expression calldata:

```shell
$ sncast --account myuser \
    call \
    --url http://127.0.0.1:5050 \
    --contract-address 0x016ad425af4585102e139d4fb2c76ce786d1aaa1cfcd88a51f3ed66601b23cdd \
    --function tuple_fn \
    --calldata (0x10, 3, data_stransformer_contract::Enum::One) \
    --block-id latest
```

getting the same result.

> 📝 **Note**
> All data types are serialized according to the official [Starknet specification](https://docs.starknet.io/architecture-and-concepts/smart-contracts/serialization-of-cairo-types/).

> 📝 **Note**
> User-defined items such as enums and structs should be referred to depending on a way they are defined in ABI.\
> In general, paths to items have form: `<project-name>::<module-path>::<item-name>`.

## Supported Expressions

> 💡 **Info**
> Only **constant** expressions are supported. Defining and referencing variables and calling functions (either builtin, user-defined or external) is not allowed.

Cast supports most important Cairo corelib types:
  * `bool`
  * signed integers (`i8`, `i16`, `i32`, `i64`, `i128`)
  * unsigned integers (`u8`, `u16`, `u32`, `u64`, `u96`, `u128`, `u256`, `u384`, `u512`)
  * `felt252` (numeric literals and so-called *shortstrings*)
  * `ByteArray`
  * `ContractAddress`
  * `ClassHash`
  * `StorageAddress`
  * `EthAddress`
  * `bytes31`
  * `Array` - using `array![]` macro

Numeric types (primitives and `felt252`) can be paseed with type suffix specified -\
for example `--calldata (420_u64,)`.

## More Complex Examples

  1. `complex_fn` - different data types:

  ```shell
  $ sncast --account myuser \
      call \
      --url http://127.0.0.1:5050 \
      --contract-address 0x016ad425af4585102e139d4fb2c76ce786d1aaa1cfcd88a51f3ed66601b23cdd \
      --function complex_fn \
      --calldata (\
          array![array![1, 2], array![3, 4, 5], array![6]], \
          12, \
          -128_i8, \
          "Some string (a ByteArray)", \
          ('a shortstring', 32_u32), \
          true, \
          0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff \
      ) \
      --block-id latest
  ```

  Note the commas separating the arguments.

  2. `nested_struct_fn` - struct nesting:

  ```shell
  $ sncast --account myuser \
      call \
      --url http://127.0.0.1:5050 \
      --contract-address 0x016ad425af4585102e139d4fb2c76ce786d1aaa1cfcd88a51f3ed66601b23cdd \
      --function nested_struct_fn \
      --calldata (\
          data_transformer_contract::NestedStructWithField { \
              a: data_transformer_contract::SimpleStruct { a: 10 }, \
              b: 12 \
          } \
      ) \
      --block-id latest
  ```
