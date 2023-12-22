#[test]
#[fork(url: "https://test.com")]
fn incorrect_fork_attributes() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Number(Latest))]
fn incorrect_fork_attributes2() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Number(19446744073709551615))]
fn incorrect_fork_attributes3() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Hash(Random))]
fn incorrect_fork_attributes4() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Hash(Latest))]
fn incorrect_fork_attributes5() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Tag(12345))]
fn incorrect_fork_attributes6() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Tag(0x12345))]
fn incorrect_fork_attributes7() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Tag(Random))]
fn incorrect_fork_attributes8() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Number(Random))]
fn incorrect_fork_attributes9() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: Number(12345))]
fn incorrect_fork_attributes10() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: Hash(0x12345))]
fn incorrect_fork_attributes11() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: Tag(Latest))]
fn incorrect_fork_attributes12() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockWhat::Number(12345))]
fn incorrect_fork_attributes13() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: Something::BlockId::Number(12345))]
fn incorrect_fork_attributes14() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(
    url: "http://188.34.188.184:9545/rpc/v0_6",
    block_id: BlockId::Tag(xddd::d00pa::hehe::BlockTag::Latest)
)]
fn incorrect_fork_attributes15() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Tag(sumting::Latest))]
fn incorrect_fork_attributes16() {
    assert(1 == 1, 'ok')
}
