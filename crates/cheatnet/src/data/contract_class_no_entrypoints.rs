use indoc::indoc;

pub const NO_ENTRYPOINTS_CASM: &str = indoc!(
    r#"{
          "prime": "0x800000000000011000000000000000000000000000000000000000000000001",
          "compiler_version": "2.4.0",
          "bytecode": [],
          "hints": [],
          "entry_points_by_type": {
            "EXTERNAL": [],
            "L1_HANDLER": [],
            "CONSTRUCTOR": []
          }
        }"#
);
