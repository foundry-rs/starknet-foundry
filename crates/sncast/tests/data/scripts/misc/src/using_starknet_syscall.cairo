use core::box::BoxTrait;
use starknet::get_execution_info;

fn main() {
    let exec_info = get_execution_info().unbox();
    assert(1 == 2, 'unreachable');
}
