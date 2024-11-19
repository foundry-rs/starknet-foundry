#[cfg(test)]
mod tests {
    use super::panicking_function;

    #[test]
    //ANCHOR_END:first_half
    #[should_panic(expected: 'aaa')]
    //ANCHOR:second_half
    fn failing() {
        panicking_function();
        assert(2 == 2, '2 == 2');
    }
}

mod dummy {} // trick `scarb fmt --check`
