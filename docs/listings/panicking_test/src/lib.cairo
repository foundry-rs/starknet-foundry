fn panicking_function() {
    let mut data = array![];
    data.append('panic message');
    panic(data)
}

#[cfg(test)]
mod tests {
    use super::panicking_function;

    #[test]
    fn failing() {
        panicking_function();
        assert(2 == 2, '2 == 2');
    }
}
