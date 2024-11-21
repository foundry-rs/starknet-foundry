//ANCHOR:first_half
fn panicking_function() {
    let mut data = array![];
    data.append('panic message');
    panic(data)
}

#[cfg(test)]
mod tests {
    use super::panicking_function;

    #[test]
    //ANCHOR_END:first_half
    //ANCHOR:second_half
    fn failing() {
        panicking_function();
        assert(2 == 2, '2 == 2');
    }
}
//ANCHOR_END:second_half


