use hello_world_script::some_code::calc_fib;

fn main() {
    let res = calc_fib();
    assert(res == 5, res);
}
