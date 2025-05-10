// Tests for NonZeroUsize and CheatSpan
use snforge_std::cheatcodes::{NonZeroUsize, CheatSpan};

#[test]
fn test_non_zero_usize_new_panics_on_zero() {
    let panicked = core::panic::catch_unwind(|| {
        let _ = NonZeroUsize::new(0);
    })
    assert(panicked.is_err(), 'NonZeroUsize::new(0) should panic');
}

#[test]
fn test_non_zero_usize_decrement() {
    let nzu = NonZeroUsize::new(3);
    let nzu2 = nzu.decrement().unwrap();
    assert(nzu2.value == 2, 'Decrement failed');
    let nzu1 = nzu2.decrement().unwrap();
    assert(nzu1.value == 1, 'Decrement failed');
    let none = nzu1.decrement();
    assert(none.is_none(), 'Should be None when decrementing 1');
}

#[test]
fn test_cheat_span_target_calls_zero_panics() {
    let panicked = core::panic::catch_unwind(|| {
        let _ = CheatSpan::TargetCalls(NonZeroUsize::new(0));
    });
    assert(panicked.is_err(), 'CheatSpan::TargetCalls(0) should panic');
}

#[test]
fn test_cheat_span_target_calls_valid() {
    let cs = CheatSpan::target_calls(5);
    match cs {
        CheatSpan::TargetCalls(nzu) => assert_eq!(nzu.0.get(), 5),
        _ => panic!("Expected TargetCalls variant"),
    }
}
