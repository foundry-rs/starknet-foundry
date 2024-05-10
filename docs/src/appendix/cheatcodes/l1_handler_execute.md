# `l1_handler_execute`

> `fn new(target: ContractAddress, selector: felt252) -> L1Handler`
Returns a structure referring to a L1 handler function.

> `fn execute(self: L1Handler) -> SyscallResult<()>`
Mocks a L1 -> L2 message from Ethereum handled by the given L1 handler function.
