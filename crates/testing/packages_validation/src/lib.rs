use camino::Utf8PathBuf;
use scarb_api::ScarbCommand;
use std::process::Stdio;

pub fn check_and_lint(package_path: &Utf8PathBuf) {
    check_and_lint_with_envs(package_path, &[]);
}

pub fn check_and_lint_with_envs(package_path: &Utf8PathBuf, envs: &[(&str, &str)]) {
    let check_output = ScarbCommand::new()
        .envs(envs.iter().copied())
        .current_dir(package_path)
        .arg("check")
        .command()
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to run `scarb check`");
    assert!(
        check_output.status.success(),
        "`scarb check` failed in {package_path}",
    );

    let lint_output = ScarbCommand::new()
        .envs(envs.iter().copied())
        .current_dir(package_path)
        .arg("lint")
        .command()
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to run `scarb lint`");
    assert!(
        lint_output.status.success(),
        "`scarb lint` failed in {package_path}"
    );
}
