use forge::{ExitStatus, main_execution};
use foundry_ui::{Printer, UI, components::error::ErrorMessage};

fn main() {
    let ui = UI::default();
    match main_execution(&ui) {
        Ok(ExitStatus::Success) => std::process::exit(0),
        Ok(ExitStatus::Failure) => std::process::exit(1),
        Err(error) => {
            ui.println(&ErrorMessage::from(error));
            std::process::exit(2);
        }
    };
}
