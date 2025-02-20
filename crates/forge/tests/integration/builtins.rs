use indoc::indoc;
use test_utils::runner::assert_passed;
use test_utils::running_tests::run_test_case;

#[test]
fn builtin_test() {
    let test = test_utils::test_case!(indoc!(
        r"
        use core::{
            pedersen::Pedersen, RangeCheck, integer::Bitwise, ec::EcOp, poseidon::Poseidon,
            SegmentArena, gas::GasBuiltin, starknet::System
        };
        use core::circuit::{RangeCheck96, AddMod, MulMod};

        #[test]
        fn test_builtins() {
            core::internal::require_implicit::<Pedersen>();
            core::internal::require_implicit::<RangeCheck>();
            core::internal::require_implicit::<Bitwise>();
            core::internal::require_implicit::<EcOp>();
            core::internal::require_implicit::<Poseidon>();
            core::internal::require_implicit::<SegmentArena>();
            core::internal::require_implicit::<GasBuiltin>();
            core::internal::require_implicit::<System>();
            core::internal::require_implicit::<RangeCheck96>();
            core::internal::require_implicit::<AddMod>();
            core::internal::require_implicit::<MulMod>();
        }
        "
    ));

    let result = run_test_case(&test);

    assert_passed(&result);
}
