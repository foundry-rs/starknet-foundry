# RPC Version (starknet.rs)

If `starknet.rs` version changed, check if `rpc` version supported by it also changed.
If so update it in `./crates/forge/expected-rpc-version`.  
If you run release script (`./scripts/release.sh`) it will ask you, and update it if changed automatically.