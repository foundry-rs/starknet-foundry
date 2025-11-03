use std::{env, sync::Arc};

use crate::profile_validation::check_profile_compatibility;
use crate::run_tests::workspace::run_for_workspace;
use crate::warn::warn_if_snforge_std_does_not_match_package_version;
use crate::warn::{
    error_if_snforge_std_deprecated_missing, error_if_snforge_std_deprecated_not_compatible,
    error_if_snforge_std_not_compatible, warn_if_backtrace_without_panic_hint,
    warn_if_snforge_std_deprecated_does_not_match_package_version,
};
use crate::{ColorOption, ExitStatus, MINIMAL_SCARB_VERSION_FOR_V2_MACROS_REQUIREMENT, TestArgs};
use anyhow::Result;
use foundry_ui::UI;
use scarb_api::metadata::metadata_with_opts;
use scarb_api::{ScarbCommand, metadata::MetadataOpts};
use scarb_metadata::Metadata;

#[tracing::instrument(skip_all, level = "debug")]
pub async fn test(args: TestArgs, ui: Arc<UI>) -> Result<ExitStatus> {
    set_color_envs(&args.color);

    let scarb_metadata = metadata_with_opts(MetadataOpts {
        profile: args.scarb_args.profile.specified(),
        ..MetadataOpts::default()
    })?;

    check_profile_compatibility(&args, &scarb_metadata)?;

    warn_if_scarb_version_not_compatible(&scarb_metadata, &ui)?;
    warn_if_backtrace_without_panic_hint(&scarb_metadata, &ui);

    run_for_workspace(&scarb_metadata, args, ui).await
}

fn set_color_envs(color: &ColorOption) {
    match color {
        // SAFETY: This runs in a single-threaded environment.
        ColorOption::Always => unsafe { env::set_var("CLICOLOR_FORCE", "1") },
        // SAFETY: This runs in a single-threaded environment.
        ColorOption::Never => unsafe { env::set_var("CLICOLOR", "0") },
        ColorOption::Auto => (),
    }
}

fn warn_if_scarb_version_not_compatible(scarb_metadata: &Metadata, ui: &Arc<UI>) -> Result<()> {
    let scarb_version = ScarbCommand::version().run()?.scarb;

    if scarb_version >= MINIMAL_SCARB_VERSION_FOR_V2_MACROS_REQUIREMENT {
        error_if_snforge_std_not_compatible(&scarb_metadata)?;
        warn_if_snforge_std_does_not_match_package_version(&scarb_metadata, &ui)?;
    } else {
        error_if_snforge_std_deprecated_missing(&scarb_metadata)?;
        error_if_snforge_std_deprecated_not_compatible(&scarb_metadata)?;
        warn_if_snforge_std_deprecated_does_not_match_package_version(&scarb_metadata, &ui)?;
    }

    Ok(())
}
