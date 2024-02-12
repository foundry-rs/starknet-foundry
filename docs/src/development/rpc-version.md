# RPC Version (starknet.rs)

If `starknet.rs` version changed, check if `rpc` version supported by it also changed.
If so update 
```rs
shared::consts::EXPECTED_RPC_VERSION;
```

