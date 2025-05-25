use forge::{ExitStatus, main_execution};
use foundry_ui::UI;

fn main() {
    let ui = UI::default();
    match main_execution(&ui) {
        Ok(ExitStatus::Success) => std::process::exit(0),
        Ok(ExitStatus::Failure) => std::process::exit(1),
        Err(error) => {
            ui.print_error(&format!("{error:#}"));
            std::process::exit(2);
        }
    };
}
