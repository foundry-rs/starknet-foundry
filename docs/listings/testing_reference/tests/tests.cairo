use snforge_std::testing::get_current_vm_step;

#[feature("safe_dispatcher")]
fn setup() {
    let mut _counter = 0_u32;

    while _counter < 1_000 {
        _counter += 1;
    }
}

#[test]
fn test_setup_steps() {
    let steps_start = get_current_vm_step();
    setup();
    let steps_end = get_current_vm_step();

    // Assert that setup used no more than 20_000 steps
    assert!(steps_end - steps_start <= 20_000);
}
