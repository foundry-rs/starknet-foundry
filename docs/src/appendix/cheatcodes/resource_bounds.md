# `resource_bounds`

Cheatcodes modifying `resource_bounds`:

## `cheat_resource_bounds`
> `fn cheat_resource_bounds(contract_address: ContractAddress, resource_bounds: Span<ResourceBounds>, span: CheatSpan)`

Changes the transaction resource bounds for the given target and span.

## `start_cheat_resource_bounds_global`
> `fn start_cheat_resource_bounds_global(resource_bounds: Span<ResourceBounds>)`

Changes the transaction resource bounds for all targets.

## `start_cheat_resource_bounds`
> `fn start_cheat_resource_bounds(contract_address: ContractAddress, resource_bounds: Span<ResourceBounds>)`

Changes the transaction resource bounds for the given target.

## `stop_cheat_resource_bounds`
> `fn stop_cheat_resource_bounds(contract_address: ContractAddress)`

Cancels the `cheat_resource_bounds` / `start_cheat_resource_bounds` for the given target.

## `stop_cheat_resource_bounds_global`
> `fn stop_cheat_resource_bounds_global()`

Cancels the `start_cheat_resource_bounds_global`.
