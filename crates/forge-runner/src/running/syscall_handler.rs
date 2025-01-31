use blockifier::{
    execution::{
        entry_point::EntryPointExecutionContext, execution_utils::ReadOnlySegments,
        syscalls::hint_processor::SyscallHintProcessor,
    },
    state::state_api::State,
};
use cairo_lang_casm::hints::Hint;
use cairo_lang_sierra::extensions::NoGenericArgsGenericType;
use cairo_lang_sierra::{extensions::segment_arena::SegmentArenaType, ids::GenericTypeId};
use cairo_vm::{types::relocatable::Relocatable, vm::runners::cairo_runner::ExecutionResources};
use cheatnet::constants::build_test_entry_point;
use std::collections::HashMap;
use std::default::Default;

pub fn build_syscall_handler<'a>(
    blockifier_state: &'a mut dyn State,
    string_to_hint: &'a HashMap<String, Hint>,
    execution_resources: &'a mut ExecutionResources,
    context: &'a mut EntryPointExecutionContext,
    test_param_types: &[(GenericTypeId, i16)],
) -> SyscallHintProcessor<'a> {
    // Segment arena is allocated conditionally, so segment index is automatically moved (+2 segments)
    let segment_index = if test_param_types
        .iter()
        .any(|(ty, _)| ty == &SegmentArenaType::ID)
    {
        16
    } else {
        14
    };

    let entry_point = build_test_entry_point();

    SyscallHintProcessor::new(
        blockifier_state,
        context,
        Relocatable {
            segment_index,
            offset: 0,
        },
        entry_point,
        string_to_hint,
        ReadOnlySegments::default(),
    )
}
