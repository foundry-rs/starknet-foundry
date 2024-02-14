use sncast_std::{declare, DeclareResult, ScriptCommandError, ProviderError, StarknetError};

fn main() {
    let declare_result = declare('Mapaaaa', Option::None, Option::None).unwrap_err();
    println!("{:?}", declare_result);
}

