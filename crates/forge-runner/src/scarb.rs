use anyhow::Result;
use cairo_lang_sierra::program::VersionedProgram;
use camino::Utf8Path;
use scarb_api::{metadata::PackageMetadata, test_targets_by_name};
use std::{fs, io::ErrorKind};

use crate::package_tests::{TestTargetLocation, raw::TestTargetRaw};

#[tracing::instrument(skip_all, level = "debug")]
pub fn load_test_artifacts(
    target_dir: &Utf8Path,
    package: &PackageMetadata,
) -> Result<Vec<TestTargetRaw>> {
    let mut targets = vec![];

    let dedup_targets = test_targets_by_name(package);

    for (target_name, target) in dedup_targets {
        let tests_location =
            if target.params.get("test-type").and_then(|v| v.as_str()) == Some("unit") {
                TestTargetLocation::Lib
            } else {
                TestTargetLocation::Tests
            };

        let target_file = format!("{target_name}.test.sierra.json");
        let sierra_program_path = target_dir.join(target_file);

        match fs::read_to_string(&sierra_program_path) {
            Ok(value) => {
                let versioned_program = serde_json::from_str::<VersionedProgram>(&value)?;

                let sierra_program = match versioned_program {
                    VersionedProgram::V1 { program, .. } => program,
                };

                let test_target = TestTargetRaw {
                    sierra_program,
                    sierra_program_path,
                    tests_location,
                };

                targets.push(test_target);
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => Err(err)?,
        }
    }

    Ok(targets)
}
