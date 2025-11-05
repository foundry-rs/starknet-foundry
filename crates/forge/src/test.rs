use std::{env, sync::Arc};

use crate::profile_validation::check_profile_compatibility;
use crate::run_tests::workspace::run_for_workspace;
use crate::warn::warn_if_backtrace_without_panic_hint;
use crate::{ColorOption, ExitStatus, TestArgs};
use anyhow::Result;
use foundry_ui::UI;
use scarb_api::metadata::MetadataOpts;
use scarb_api::metadata::metadata_with_opts;

#[tracing::instrument(skip_all, level = "debug")]
pub async fn test(args: TestArgs, ui: Arc<UI>) -> Result<ExitStatus> {
    set_color_envs(&args.color);

    let scarb_metadata = metadata_with_opts(MetadataOpts {
        profile: args.scarb_args.profile.specified(),
        ..MetadataOpts::default()
    })?;

    check_profile_compatibility(&args, &scarb_metadata)?;

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
