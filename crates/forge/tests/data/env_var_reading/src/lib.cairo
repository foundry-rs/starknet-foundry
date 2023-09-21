use snforge_std::read_env_var;

#[test]
fn reading_env_vars() {
    let felt252_value = read_env_var('FELT_ENV_VAR');
    let short_string_value = read_env_var('STRING_ENV_VAR');

    assert(felt252_value == 987654321, 'invalid felt value');
    assert(short_string_value == 'abcde', 'invalid short string value');
}