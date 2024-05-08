# `l1_handler_execute`

> `fn new(target: ContractAddress, selector: felt252) -> L1Handler`

`target` - The target starknet contract address
`selector` - Selector of a `#[l1_handler]` function

Returns a structure referring to a L1 handler function.

> `fn execute(self: L1Handler) -> SyscallResult<()>`

`self` - `L1Handler` structure referring to a L1 handler function
`from_address` - Ethereum address of the contract that you want to a emulate message from
`payload` - The message payload that may contain any Cairo data structure that can be serialized with

Mocks a L1 -> L2 message from Ethereum handled by the given L1 handler function.
