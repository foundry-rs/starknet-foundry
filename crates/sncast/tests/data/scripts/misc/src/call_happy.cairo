use sncast_std::{call, CallResult};

fn main() {
    let eth = 0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7;
    let addr = 0x0089496091c660345BaA480dF76c1A900e57cf34759A899eFd1EADb362b20DB5;
    let call_result = call(eth.try_into().unwrap(), 'allowance', array![addr, addr]);
    let call_result = *call_result.data[0];
    assert(call_result == 0, call_result);

    let call_result = call(eth.try_into().unwrap(), 'decimals', array![]);
    let call_result = *call_result.data[0];
    assert(call_result == 18, call_result);
}
