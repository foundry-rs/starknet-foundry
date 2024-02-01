use sncast_std::{get_nonce, declare, DeclareResult, ScriptCommandError, RPCError, StarknetError, ScriptCommandErrorTrait};
use core::debug::PrintTrait;

fn main() {
    let max_fee = 1;

    let declare_nonce = get_nonce('latest');
    let declare_result = declare('Mapa', Option::Some(max_fee), Option::Some(declare_nonce)).unwrap_err();
    declare_result.print();

    assert(
        ScriptCommandError::RPCError(
            RPCError::StarknetError(StarknetError::InsufficientMaxFee)
        ) == declare_result,
        'ohno'
    )
}
