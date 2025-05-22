use forge::{ExitStatus, main_execution};
use foundry_ui::Ui;

fn main() {
    let ui = Ui::default();
    match main_execution(&ui) {
        Ok(ExitStatus::Success) => std::process::exit(0),
        Ok(ExitStatus::Failure) => std::process::exit(1),
        Err(error) => {
            ui.print_error(&error.to_string());
            std::process::exit(2);
        }
    };
}
