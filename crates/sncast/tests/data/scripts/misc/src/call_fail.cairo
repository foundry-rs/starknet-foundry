use sncast_std::{call, CallResult};

fn main() {
    let strk = 0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d;
    let call_result = call(strk.try_into().unwrap(), selector!("gimme_money"), array![]);
    call_result.expect('call failed');
}
