#[test]
#[fork(url: "https://test.com")]
fn incorrect_fork_attributes() {
    assert(1 == 1, 'ok')
}

#[test]
#[fork(url: "unparsable_url", block_id: BlockId::Number(1))]
fn unparsable_url() {
    assert(1 == 1, 'ok');
}
