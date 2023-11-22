use starknet::info::get_execution_info;
use box::BoxTrait;

fn main() {
    let exec_info = get_execution_info().unbox();
    assert(1 == 2, 'unreachable');
}
