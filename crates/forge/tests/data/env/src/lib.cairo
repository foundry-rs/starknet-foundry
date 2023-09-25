use snforge_std::env::var;

#[test]
fn reading_env_vars() {
    let felt252_value = var('FELT_ENV_VAR');
    let short_string_value = var('STRING_ENV_VAR');

    assert(felt252_value == 987654321, 'invalid felt value');
    assert(short_string_value == 'abcde', 'invalid short string value');
}
