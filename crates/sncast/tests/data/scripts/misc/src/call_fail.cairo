use sncast_std::{call, CallResult, ScriptCommandError, ScriptCommandErrorTrait};
use core::debug::PrintTrait;

fn main() {
    let eth = 0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7;
    let call_err: ScriptCommandError = call(eth.try_into().expect('deded'), 'gimme_money', array![]).unwrap_err();

    call_err.print();
    let mut arr = ArrayTrait::new();
    call_err.serialize(ref arr);
    arr.print();

    assert(ScriptCommandError::SNCastError == call_err, 'ohno')
}
