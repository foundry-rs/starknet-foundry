use sncast_std::get_nonce;

fn main() {
    let nonce = get_nonce('latest');
    println!("nonce: {}", nonce);
    println!("debug nonce: {:?}", nonce);
}
