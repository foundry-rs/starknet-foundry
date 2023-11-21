use indoc::indoc;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_gas, assert_passed, test_case};

#[test]
fn test_keccak_cost() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn test_keccak_builtin() {
                keccak::keccak_u256s_le_inputs(array![1].span());
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "test_keccak_builtin", 20.48);
}

#[test]
fn test_range_check_cost() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn test_range_check() {
                assert(1_u8 >= 1_u8, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "test_range_check", 0.16);
}

#[test]
fn test_bitwise_cost() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn test_bitwise() {
                let bitwise = 1_u8 & 1_u8;
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "test_bitwise", 0.64);
}

#[test]
fn test_pedersen_cost() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn test_pedersen() {
                core::pedersen::pedersen(1, 2);
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "test_pedersen", 0.32);
}

#[test]
fn test_poseidon_cost() {
    let test = test_case!(indoc!(
        r"
            extern type Poseidon;

            extern fn hades_permutation(
                s0: felt252, s1: felt252, s2: felt252
            ) -> (felt252, felt252, felt252) implicits(Poseidon) nopanic;

            #[test]
            fn test_poseidon() {
                hades_permutation(0, 0, 0);
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "test_poseidon", 0.32);
}

#[test]
fn test_ec_op_cost() {
    let test = test_case!(indoc!(
        r"
            use core::{ec, ec::{EcPoint, EcPointTrait}};

            #[test]
            fn test_ec_op() {
                EcPointTrait::new_from_x(1).unwrap().mul(2);
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "test_ec_op", 10.24);
}
