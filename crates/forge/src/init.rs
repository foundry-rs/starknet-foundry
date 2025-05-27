use crate::{NewArgs, Template, new};
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use foundry_ui::UI;

pub fn init(project_name: &str, ui: &UI) -> Result<()> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    let project_path = Utf8PathBuf::from_path_buf(current_dir)
        .expect("Failed to create Utf8PathBuf for the current directory")
        .join(project_name);

    // To prevent printing this warning when running scarb init/new with an older version of Scarb
    if !project_path.join("Scarb.toml").exists() {
        ui.print_warning("Command `snforge init` is deprecated and will be removed in the future. Please use `snforge new` instead.");
    }

    new::new(NewArgs {
        path: project_path,
        name: Some(project_name.to_string()),
        no_vcs: false,
        overwrite: true,
        template: Template::BalanceContract,
    })
}
