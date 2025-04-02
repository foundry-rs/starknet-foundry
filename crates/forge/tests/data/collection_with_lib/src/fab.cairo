pub mod fab_impl;

pub fn fn_from_above() -> felt252 {
    1
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_simple() {
        assert(1 == 1, 1);
    }
}
