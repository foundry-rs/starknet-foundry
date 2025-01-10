#[cfg(feature: 'enable_for_tests')]
fn foo() -> u32 {
    2
}

#[cfg(feature: 'enable_for_tests')]
#[cfg(test)]
mod tests {
    #[test]
    fn test_using_conditionally_compiled_function() {
        assert_eq!(foo(), 2);
    }
}
