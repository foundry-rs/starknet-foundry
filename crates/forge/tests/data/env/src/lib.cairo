#[cfg(test)]
mod tests {
    use snforge_std::env::var;

    #[test]
    fn reading_env_vars() {
        let felt252_value = var("FELT_ENV_VAR");
        let short_string_value = var("STRING_ENV_VAR");
        let mut byte_array_value = var("BYTE_ARRAY_ENV_VAR").span();

        assert(felt252_value == array![987654321], 'invalid felt value');
        assert(short_string_value == array!['abcde'], 'invalid short string value');

        let byte_array = Serde::<ByteArray>::deserialize(ref byte_array_value).unwrap();
        assert(
            byte_array == "that is a very long environment variable that would normally not fit",
            'Invalid ByteArray value'
        )
    }
}
