#[test]
#[fork(url: "https://test.com")]
fn incorrect_fork_attributes() {
    assert(1 == 1, 'ok')
}
