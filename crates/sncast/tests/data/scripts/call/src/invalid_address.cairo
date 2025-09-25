use sncast_std::{ProviderError, ScriptCommandError, StarknetError, call};

fn main() {
    let eth = 0x049;
    let call_err: ScriptCommandError = call(
        eth.try_into().expect('bad address'), selector!("decimals"), array![],
    )
        .unwrap_err();

    println!("{:?}", call_err);

    assert(
        ScriptCommandError::ProviderError(
            ProviderError::StarknetError(StarknetError::ContractNotFound),
        ) == call_err,
        'ohno',
    )
}
