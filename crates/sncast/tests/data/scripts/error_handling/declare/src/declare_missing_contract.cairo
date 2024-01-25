use sncast_std::{declare, DeclareResult, ScriptCommandError, RPCError, StarknetError, ScriptCommandErrorTrait};
use core::debug::PrintTrait;

fn main() {
    let declare_result = declare('Mapaaaa', Option::None, Option::None).unwrap_err();
    declare_result.print();

    assert(
        ScriptCommandError::ContractArtifactsNotFound == declare_result,
        'ohno'
    )
}

