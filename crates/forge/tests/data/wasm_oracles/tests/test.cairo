mod wasm_oracle {
    pub fn add(left: u64, right: u64) -> oracle::Result<u64> {
        oracle::invoke("wasm:wasm_oracle.wasm", "add", (left, right))
    }

    pub fn err() -> oracle::Result<Result<ByteArray, ByteArray>> {
        oracle::invoke("wasm:wasm_oracle.wasm", "err", ())
    }
}

#[test]
fn add() {
    assert!(wasm_oracle::add(2, 3) == Ok(5));
}

#[test]
fn err() {
    assert!(wasm_oracle::err() == Ok(Err("failed hard")));
}
