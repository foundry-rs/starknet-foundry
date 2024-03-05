use sncast_std::{call, CallResult};

fn main() {
    let eth = 0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7;
    let call_result = call(eth.try_into().unwrap(), selector!("gimme_money"), array![]);
    call_result.expect('call failed');
}
