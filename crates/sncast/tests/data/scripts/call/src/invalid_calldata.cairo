use sncast_std::{call, ScriptCommandError};

fn main() {
    let eth = 0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7;
    let call_err: ScriptCommandError = call(
        eth.try_into().expect('bad address'),
        selector!("allowance"),
        array![0x12, 0x12, 0x12, 0x12, 0x12],
    )
        .unwrap_err();

    println!("{:?}", call_err);

    let call_err: ScriptCommandError = call(
        eth.try_into().expect('bad address'), selector!("allowance"), array![0x12],
    )
        .unwrap_err();

    println!("{:?}", call_err);
}
