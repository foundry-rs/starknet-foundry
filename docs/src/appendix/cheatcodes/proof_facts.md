# `proof_facts`

Cheatcodes modifying `proof_facts`:

## `cheat_proof_facts`
> `fn cheat_proof_facts(contract_address: ContractAddress, proof_facts: Span<felt252>, span: CheatSpan)`

Changes the transaction proof facts for the given target and span.

## `start_cheat_proof_facts_global`
> `fn start_cheat_proof_facts_global(proof_facts: Span<felt252>)`

Changes the transaction proof facts for all targets.

## `start_cheat_proof_facts`
> `fn start_cheat_proof_facts(contract_address: ContractAddress, proof_facts: Span<felt252>)`

Changes the transaction proof facts for the given target.

## `stop_cheat_proof_facts`
> `fn stop_cheat_proof_facts(contract_address: ContractAddress)`

Cancels the `cheat_proof_facts` / `start_cheat_proof_facts` for the given target.

## `stop_cheat_proof_facts_global`
> `fn stop_cheat_proof_facts_global()`

Cancels the `start_cheat_proof_facts_global`.
