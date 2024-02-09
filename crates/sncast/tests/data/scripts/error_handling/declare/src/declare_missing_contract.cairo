use sncast_std::{declare, DeclareResult, ScriptCommandError, RPCError, StarknetError};


fn main() {
    let declare_result = declare('Mapaaaa', Option::None, Option::None).unwrap_err();
    println!("{:?}", declare_result);

    assert(
        ScriptCommandError::ContractArtifactsNotFound == declare_result,
        'ohno'
    )
}

