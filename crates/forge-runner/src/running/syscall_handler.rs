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
use cairo_vm::types::relocatable::Relocatable;
use cheatnet::constants::build_test_entry_point;
use std::collections::HashMap;
use std::default::Default;

pub fn build_syscall_handler<'a>(
    blockifier_state: &'a mut dyn State,
    string_to_hint: &'a HashMap<String, Hint>,
    context: &'a mut EntryPointExecutionContext,
    test_param_types: &[(GenericTypeId, i16)],
    builtins_len: usize,
) -> SyscallHintProcessor<'a> {
    // * Segment arena is allocated conditionally, so segment index is automatically moved (+2 segments)
    // * Each used builtin moves the offset by +1
    // * Line `let mut builtin_offset = 3;` in `create_entry_code_from_params`
    // * TODO Where is remaining +2 in base offset coming from? Maybe System builtin and Gas builtin which seem to be always included
    // TODO(#2954)
    let base_offset = 5;
    let segment_index = if test_param_types
        .iter()
        .any(|(ty, _)| ty == &SegmentArenaType::ID)
    {
        // FIXME verify this
        base_offset + builtins_len + 2
    } else {
        // FIXME verify this
        base_offset + builtins_len
    };

    let entry_point = build_test_entry_point();

    SyscallHintProcessor::new(
        blockifier_state,
        context,
        Relocatable {
            segment_index: segment_index
                .try_into()
                .expect("Failed to convert index to isize"),
            offset: 0,
        },
        entry_point,
        string_to_hint,
        ReadOnlySegments::default(),
    )
}
