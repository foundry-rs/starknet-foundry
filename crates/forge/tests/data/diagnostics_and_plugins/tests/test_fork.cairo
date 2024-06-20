#[test]
#[fork(url: "https://test.com")]
fn incorrect_fork_attributes() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "{{ NODE_RPC_URL }}", block_tag: latest)]
fn incorrect_fork_attributes2() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "{{ NODE_RPC_URL }}", block_number: 19446744073709551615)]
fn incorrect_fork_attributes3() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "{{ NODE_RPC_URL }}", block_hash: Random)]
fn incorrect_fork_attributes4() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "{{ NODE_RPC_URL }}", block_hash: Latest)]
fn incorrect_fork_attributes5() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "{{ NODE_RPC_URL }}", block_tag: 12345)]
fn incorrect_fork_attributes6() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "{{ NODE_RPC_URL }}", block_tag: 0x12345)]
fn incorrect_fork_attributes7() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "{{ NODE_RPC_URL }}", block_tag: Random)]
fn incorrect_fork_attributes8() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "{{ NODE_RPC_URL }}", block_number: Random)]
fn incorrect_fork_attributes9() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "{{ NODE_RPC_URL }}", block_number: 12345)]
fn incorrect_fork_attributes10() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "{{ NODE_RPC_URL }}", block_hash: 0x12345)]
fn incorrect_fork_attributes11() {
    assert(1 == 1, 'ok')
}
