# `get_nonce`

> `pub fn get_nonce(block_tag: felt252) -> felt252`

Gets nonce of an account for a given block tag (`pre_confirmed` or `latest`) and returns nonce as `felt252`.

- `block_tag` - block tag name, one of `pre_confirmed` or `latest`.

```rust
{{#include ../../../listings/get_nonce/src/lib.cairo}}
```
