use core::panics::panic_with_byte_array;

mod wasm_oracle {
    pub fn add(left: u64, right: u64) -> oracle::Result<u64> {
        oracle::invoke("wasm:wasm_oracle.wasm", "add", (left, right))
    }

    pub fn err() -> oracle::Result<Result<ByteArray, ByteArray>> {
        oracle::invoke("wasm:wasm_oracle.wasm", "err", ())
    }

    pub fn panic() -> oracle::Result<Result<ByteArray, ByteArray>> {
        oracle::invoke("wasm:wasm_oracle.wasm", "panic", ())
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

#[should_panic]
#[test]
fn panic() {
    wasm_oracle::panic().unwrap().unwrap();
}

#[test]
fn unexpected_panic() {
    wasm_oracle::panic().unwrap().unwrap();
}

#[test]
fn panic_contents() {
    let err = wasm_oracle::panic().unwrap_err();
    // Panic with error so we get better error than "unwrap failed"
    panic_with_byte_array(@format!("{}", err))
}
