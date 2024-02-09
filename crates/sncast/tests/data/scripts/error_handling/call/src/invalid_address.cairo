use sncast_std::{
    call, CallResult, ScriptCommandError, RPCError, StarknetError
};


fn main() {
    let eth = 0x049;
    let call_err: ScriptCommandError = call(
        eth.try_into().expect('bad address'), 'decimals', array![]
    )
        .unwrap_err();

    println!("{:?}", call_err);

    assert(
        ScriptCommandError::RPCError(
            RPCError::StarknetError(StarknetError::ContractNotFound)
        ) == call_err,
        'ohno'
    )
}
