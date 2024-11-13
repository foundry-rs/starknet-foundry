# `get_nonce`

> `pub fn get_nonce(block_tag: felt252) -> felt252`

Gets nonce of an account for a given block tag (`pending` or `latest`) and returns nonce as `felt252`.

- `block_tag` - block tag name, one of `pending` or `latest`.

```rust
{{#include ../../../listings/sncast_library/scripts/get_nonce/src/lib.cairo}}
```
