#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0.4", block_id: BlockId::Number(19446744073709551615))]
fn incorrect_fork_attributes3() {
    assert(1 == 1, 'ok')
}
