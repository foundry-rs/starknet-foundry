use forge::{ExitStatus, main_execution, pretty_printing};
use foundry_ui::Ui;

fn main() {
    let ui = Ui::default();
    match main_execution(&ui) {
        Ok(ExitStatus::Success) => std::process::exit(0),
        Ok(ExitStatus::Failure) => std::process::exit(1),
        Err(error) => {
            pretty_printing::print_error_message(&error);
            std::process::exit(2);
        }
    };
}
