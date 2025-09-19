use forge::{ExitStatus, main_execution};
use foundry_ui::{UI, components::error::ErrorMessage};
use std::io::IsTerminal;
use std::sync::Arc;
use std::{env, io};

fn main() {
    let _guard = init_logging();
    let ui = Arc::new(UI::default());
    match main_execution(ui.clone()) {
        Ok(ExitStatus::Success) => std::process::exit(0),
        Ok(ExitStatus::Failure) => std::process::exit(1),
        Err(error) => {
            ui.println(&ErrorMessage::from(error));
            std::process::exit(2);
        }
    };
}

fn init_logging() -> Option<impl Drop> {
    use chrono::Local;
    use std::fs;

    use std::path::PathBuf;
    use tracing_chrome::ChromeLayerBuilder;
    use tracing_subscriber::filter::{EnvFilter, LevelFilter, Targets};
    use tracing_subscriber::fmt::Layer;
    use tracing_subscriber::fmt::time::Uptime;
    use tracing_subscriber::prelude::*;

    let mut guard = None;

    let fmt_layer = Layer::new()
        .with_writer(io::stderr)
        .with_ansi(io::stderr().is_terminal())
        .with_timer(Uptime::default())
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::WARN.into())
                .with_env_var("SNFORGE_LOG")
                .from_env_lossy(),
        );

    let tracing_profile = env::var("SNFORGE_TRACING_PROFILE").ok().is_some_and(|var| {
        let s = var.as_str();
        s == "true" || s == "1"
    });

    let profile_layer = if tracing_profile {
        let mut path = PathBuf::from(format!(
            "./snforge-profile-{}.json",
            Local::now().to_rfc3339()
        ));

        // Create the file now, so that we early panic, and `fs::canonicalize` will work.
        let profile_file = fs::File::create(&path).expect("failed to create profile file");

        // Try to canonicalize the path so that it is easier to find the file from logs.
        if let Ok(canonical) = fs::canonicalize(&path) {
            path = canonical;
        }

        eprintln!(
            "`snforge` run will output tracing profile to: {}",
            path.display()
        );
        eprintln!(
            "open that file with https://ui.perfetto.dev (or chrome://tracing) to analyze it"
        );

        let (profile_layer, profile_layer_guard) = ChromeLayerBuilder::new()
            .writer(profile_file)
            .include_args(true)
            .build();

        // Filter out less important logs because they're too verbose,
        // and with them the profile file quickly grows to several GBs of data.
        let profile_layer = profile_layer.with_filter(
            Targets::new()
                .with_default(LevelFilter::TRACE)
                .with_target("salsa", LevelFilter::WARN),
        );

        guard = Some(profile_layer_guard);
        Some(profile_layer)
    } else {
        None
    };

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(profile_layer),
    )
    .expect("could not set up global logger");

    guard
}
