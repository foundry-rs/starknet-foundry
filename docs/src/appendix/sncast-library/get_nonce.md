# `get_nonce`

> `fn get_nonce(block_tag: felt252) -> felt252`

Gets nonce of an account for a given block tag (`pending` or `latest`) and returns nonce as `felt252`.

- `block_tag` - block tag name, one of `pending` or `latest`.

```rust
use sncast_std::{get_nonce};
use debug::PrintTrait;

fn main() {
    let nonce = get_nonce('latest');
    nonce.print();
}
```
