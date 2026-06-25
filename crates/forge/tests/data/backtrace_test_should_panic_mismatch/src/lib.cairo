pub fn fail_deep() {
    assert(1 != 1, 'Assert failed');
}

pub fn fail() {
    fail_deep();
}

#[cfg(test)]
mod tests {
    use super::fail;

    #[should_panic(expected: 'wrong msg')]
    #[test]
    fn test_panics_in_test_body_with_should_panic_mismatch() {
        fail();
    }
}
