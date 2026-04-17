use crate::TestArgs;
use backtrace::check_backtrace_compatibility;
use coverage::check_coverage_compatibility;
use debugger::check_debugger_compatibility;
use forge_runner::backtrace::is_backtrace_enabled;
use scarb_metadata::Metadata;
use serde_json::Value;

mod backtrace;
mod coverage;
mod debugger;

/// Checks if the compiler settings provided in [`Metadata`] can be used to run
/// coverage, backtrace and debugger if applicable.
pub fn check_compiler_config_compatibility(
    test_args: &TestArgs,
    scarb_metadata: &Metadata,
) -> anyhow::Result<()> {
    // Config of the first CU is as good as any as all compilation units have the same compiler
    // config.
    let Some(compiler_config) = scarb_metadata
        .compilation_units
        .first()
        .map(|cu| &cu.compiler_config)
    else {
        return Ok(());
    };
    let profile = &scarb_metadata.current_profile;
    let workspace_manifest_path = &scarb_metadata.workspace.manifest_path;

    if test_args.coverage {
        check_coverage_compatibility(compiler_config, profile, workspace_manifest_path)?;
    }

    if test_args.launch_debugger {
        check_debugger_compatibility(compiler_config, profile, workspace_manifest_path)?;
    }

    if is_backtrace_enabled() {
        check_backtrace_compatibility(
            test_args,
            compiler_config,
            profile,
            workspace_manifest_path,
        )?;
    }

    Ok(())
}

/// This function exists for backwards compatibility purposes.
/// `add_statements_code_locations_debug_info` and `add_statements_functions_debug_info`
/// keys in compiler config had `unstable` prefix in Scarb versions before 2.15.0-rc.0.
fn bool_field_or_unstable(compiler_config: &Value, key: &str) -> bool {
    compiler_config
        .get(key)
        .or_else(|| compiler_config.get(format!("unstable_{key}")))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn bool_field(compiler_config: &Value, key: &str) -> bool {
    compiler_config
        .get(key)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn str_field<'a>(compiler_config: &'a Value, key: &str) -> &'a str {
    compiler_config
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
}
