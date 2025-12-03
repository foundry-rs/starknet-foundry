use crate::scarb::config::ForgeConfigFromScarb;
use anyhow::{Context, Result, anyhow};
use cairo_lang_sierra::extensions::core::{CoreLibfunc, CoreType};
use cairo_lang_sierra::program::{ProgramArtifact, VersionedProgram};
use cairo_lang_sierra::program_registry::ProgramRegistry;
use cairo_lang_sierra_type_size::get_type_size_map;
use camino::Utf8Path;
use configuration::Config;
use configuration::core::Profile;
use forge_runner::package_tests::{TestCandidate, TestDetails, TestTarget, TestTargetLocation};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use scarb_api::{ScarbCommand, test_targets_by_name};
use scarb_metadata::{Metadata, PackageId, PackageMetadata};
use scarb_ui::args::{FeaturesSpec, PackagesFilter, ProfileSpec};
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::sync::Arc;
use universal_sierra_compiler_api::compile_raw_sierra_at_path;

pub mod config;

impl Config for ForgeConfigFromScarb {
    fn tool_name() -> &'static str {
        "snforge"
    }

    fn from_raw(config: serde_json::Value) -> Result<Self>
    where
        Self: Sized,
    {
        serde_json::from_value(config.clone()).context("Failed to parse snforge config")
    }
}

/// Loads config for a specific package from the `Scarb.toml` file
/// # Arguments
/// * `metadata` - Scarb metadata object
/// * `package` - Id of the Scarb package
pub fn load_package_config<T: Config + Default>(
    metadata: &Metadata,
    package: &PackageId,
) -> Result<T> {
    let maybe_raw_metadata = metadata
        .get_package(package)
        .ok_or_else(|| anyhow!("Failed to find metadata for package = {package}"))?
        .tool_metadata(T::tool_name())
        .cloned();

    match maybe_raw_metadata {
        Some(raw_metadata) => configuration::core::load_config(raw_metadata, Profile::None),
        None => Ok(T::default()),
    }
}

#[tracing::instrument(skip_all, level = "debug")]
pub fn build_artifacts_with_scarb(
    filter: PackagesFilter,
    features: FeaturesSpec,
    profile: ProfileSpec,
    no_optimization: bool,
) -> Result<()> {
    if no_optimization {
        build_contracts_with_scarb(filter.clone(), features.clone(), profile.clone())?;
    }
    build_test_artifacts_with_scarb(filter, features, profile)?;
    Ok(())
}

fn build_contracts_with_scarb(
    filter: PackagesFilter,
    features: FeaturesSpec,
    profile: ProfileSpec,
) -> Result<()> {
    ScarbCommand::new_with_stdio()
        .arg("build")
        .packages_filter(filter)
        .features(features)
        .profile(profile)
        .run()
        .context("Failed to build contracts with Scarb")?;
    Ok(())
}

fn build_test_artifacts_with_scarb(
    filter: PackagesFilter,
    features: FeaturesSpec,
    profile: ProfileSpec,
) -> Result<()> {
    ScarbCommand::new_with_stdio()
        .arg("build")
        .arg("--test")
        .packages_filter(filter)
        .features(features)
        .profile(profile)
        .run()
        .context("Failed to build test artifacts with Scarb")?;
    Ok(())
}

pub fn load_test_candidates(sierra_program: &ProgramArtifact) -> Result<Vec<TestCandidate>> {
    macro_rules! by_id {
        ($field:ident) => {{
            let temp: HashMap<_, _> = sierra_program
                .program
                .$field
                .iter()
                .map(|f| (f.id.id, f))
                .collect();

            temp
        }};
    }
    let funcs = by_id!(funcs);
    let type_declarations = by_id!(type_declarations);

    let sierra_program_registry =
        ProgramRegistry::<CoreType, CoreLibfunc>::new(&sierra_program.program)?;
    let type_size_map = get_type_size_map(&sierra_program.program, &sierra_program_registry)
        .ok_or_else(|| anyhow!("can not get type size map"))?;

    let default_executables = vec![];
    let debug_info = sierra_program.debug_info.clone();
    let executables = debug_info
        .as_ref()
        .and_then(|info| info.executables.get("snforge_internal_test_executable"))
        .unwrap_or(&default_executables);

    let test_cases = executables
        .par_iter()
        .map(|case| {
            let func = funcs[&case.id];
            let name = case.debug_name.clone().unwrap().into();
            let test_details = TestDetails::build(func, &type_declarations, &type_size_map);

            Ok(TestCandidate { name, test_details })
        })
        .collect::<Result<_>>()?;

    Ok(test_cases)
}

#[tracing::instrument(skip_all, level = "debug")]
pub fn load_test_artifacts(
    target_dir: &Utf8Path,
    package: &PackageMetadata,
) -> Result<Vec<TestTarget<TestCandidate>>> {
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
                let casm_program = Arc::new(compile_raw_sierra_at_path(
                    sierra_program_path.as_std_path(),
                )?);
                let sierra_program_path = Arc::new(sierra_program_path);

                // let test_target = TestTargetRaw {
                let test_cases = load_test_candidates(&sierra_program)?;
                let test_target = TestTarget {
                    sierra_program,
                    sierra_program_path,
                    tests_location,
                    casm_program: casm_program,
                    test_cases,
                };

                targets.push(test_target);
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => Err(err)?,
        }
    }

    Ok(targets)
}
