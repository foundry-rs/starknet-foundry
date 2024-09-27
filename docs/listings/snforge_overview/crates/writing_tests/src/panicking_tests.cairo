//ANCHOR:first_half
fn panicking_function() {
    let mut data = array![];
    data.append('aaa');
    panic(data)
}

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
//ANCHOR_END:second_half

mod dummy {} // trick `scarb fmt --check`
