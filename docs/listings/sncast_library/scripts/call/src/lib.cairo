use sncast_std::call;
use starknet::ContractAddress;

fn main() {
    let contract_address: ContractAddress =
        0x1e52f6ebc3e594d2a6dc2a0d7d193cb50144cfdfb7fdd9519135c29b67e427
        .try_into()
        .expect('Invalid contract address value');

    let result = call(contract_address, selector!("get"), array![0x1]).expect('call failed');

    println!("call result: {}", result);
    println!("debug call result: {:?}", result);
}
