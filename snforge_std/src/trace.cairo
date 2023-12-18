use starknet::testing::cheatcode;

fn print_last_transaction_trace() {
    cheatcode::<'print_last_transaction_trace'>(array![].span());
}
