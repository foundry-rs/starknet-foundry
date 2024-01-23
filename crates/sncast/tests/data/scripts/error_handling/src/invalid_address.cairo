use sncast_std::{
    call, CallResult, ScriptCommandError, RPCError, StarknetError, ScriptCommandErrorTrait
};
use core::debug::PrintTrait;

fn main() {
    let eth = 0x049;
    let call_err: ScriptCommandError = call(
        eth.try_into().expect('bad address'), 'decimals', array![]
    )
        .unwrap_err();

    call_err.print();

    assert(
        ScriptCommandError::RPCError(
            RPCError::StarknetError(StarknetError::ContractNotFound)
        ) == call_err,
        'ohno'
    )
}
