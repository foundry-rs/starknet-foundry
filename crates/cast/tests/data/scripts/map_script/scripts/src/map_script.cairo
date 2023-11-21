use sncast_std::{declare, DeclareResult};

fn main() {
    let max_fee = 99999999999999999;
    let declare_result = declare('Mapa', Option::Some(max_fee));

    let declare_result_mapa2 = declare('Mapa2', Option::None);
}
