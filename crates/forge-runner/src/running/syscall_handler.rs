use cairo_lang_sierra::extensions::NoGenericArgsGenericType;
use cairo_lang_sierra::{extensions::segment_arena::SegmentArenaType, ids::GenericTypeId};

#[must_use]
pub fn has_segment_arena(test_param_types: &[(GenericTypeId, i16)]) -> bool {
    test_param_types
        .iter()
        .any(|(ty, _)| ty == &SegmentArenaType::ID)
}

#[must_use]
pub fn syscall_handler_offset(builtins_len: usize, has_segment_arena: bool) -> usize {
    // * Segment arena is allocated conditionally, so segment index is automatically moved (+2 segments)
    // * Each used builtin moves the offset by +1
    // * Line `let mut builtin_offset = 3;` in `create_entry_code_from_params`
    // * TODO(#2967) Where is remaining +2 in base offset coming from? Maybe System builtin and Gas builtin which seem to be always included
    let base_offset = 5;
    if has_segment_arena {
        base_offset + builtins_len + 2
    } else {
        base_offset + builtins_len
    }
}
