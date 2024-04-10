# `declare`

> `fn declare(contract: ByteArray) -> Result<ContractClass, Array<felt252>>`

Declares a contract for later deployment. The `contract` is the name of the contract (part after `mod` keyword) e.g. `"HelloStarknet"`.

Check docs of [`ContractClass`](./contract_class.md) for more info about the resulting struct.
