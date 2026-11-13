use foundry_ui::{Message, OutputFormat, UI};
use serde::Serialize;
use serde_json::Value;

struct MessageWrapper<T: Message> {
    command: String,
    message: T,
}
struct ErrorWrapper<T: Message> {
    command: String,
    error: T,
}
struct NotificationWrapper<T: Message> {
    notification: T,
}
struct WarningWrapper<T: Message> {
    warning: T,
}

#[derive(Serialize)]
struct OutputWithCommand<'a, T> {
    r#type: String,
    command: String,
    #[serde(flatten)]
    data: &'a T,
}

#[derive(Serialize)]
struct Output<'a, T> {
    r#type: String,
    #[serde(flatten)]
    data: &'a T,
}

impl<T> Message for MessageWrapper<T>
where
    T: Message,
{
    fn text(&self) -> String {
        self.message.text()
    }

    fn json(&self) -> Value {
        let data = self.message.json();

        serde_json::to_value(OutputWithCommand {
            r#type: "response".to_string(),
            command: self.command.clone(),
            data: &data,
        })
        .expect("Failed to serialize message")
    }
}

impl<T> Message for ErrorWrapper<T>
where
    T: Message,
{
    fn text(&self) -> String {
        self.error.text()
    }

    fn json(&self) -> Value {
        let data = self.error.json();

        serde_json::to_value(OutputWithCommand {
            r#type: "error".to_string(),
            command: self.command.clone(),
            data: &data,
        })
        .expect("Failed to serialize message")
    }
}

impl<T> Message for WarningWrapper<T>
where
    T: Message,
{
    fn text(&self) -> String {
        self.warning.text()
    }

    fn json(&self) -> Value {
        let data = self.warning.json();

        serde_json::to_value(Output {
            r#type: "warning".to_string(),
            data: &data,
        })
        .expect("Failed to serialize message")
    }
}

impl<T> Message for NotificationWrapper<T>
where
    T: Message,
{
    fn text(&self) -> String {
        self.notification.text()
    }

    fn json(&self) -> Value {
        let data = self.notification.json();

        serde_json::to_value(Output {
            r#type: "notification".to_string(),
            data: &data,
        })
        .expect("Failed to serialize message")
    }
}

#[derive(Clone, Copy)]
enum Stream {
    Stdout,
    Stderr,
}

fn emit<T, W, F>(ui: &UI, message: T, wrap: F, stream: Stream)
where
    T: Message,
    W: Message,
    F: FnOnce(T) -> W,
{
    // TODO(#3960) Add better handling for no JSON output
    if ui.output_format() == OutputFormat::Json && message.json() == Value::Null {
        return;
    }

    let wrapped = wrap(message);

    #[expect(clippy::disallowed_methods)]
    match stream {
        Stream::Stdout => ui.println(&wrapped),
        Stream::Stderr => ui.eprintln(&wrapped),
    }
}

/// `sncast` extension to `foundry_ui::UI`
pub trait UIExt {
    /// Prints a successful response associated with `command` to standard output.
    fn print_command_message<T: Message>(&self, command: &str, message: T);

    /// Prints an error response associated with `command` to standard error.
    fn print_command_error<T: Message>(&self, command: &str, message: T);

    /// Prints a notification to standard output.
    fn print_notification<T: Message>(&self, message: T);

    /// Prints a warning to standard output.
    ///
    /// Wraps `message` with `foundry_ui::WarningMessage`
    fn print_warning<T: Message>(&self, message: T);

    /// Prints an error to standard error.
    ///
    /// Wraps `message` with `foundry_ui::ErrorMessage`
    fn print_error<T: Message>(&self, message: T);
}

impl UIExt for UI {
    fn print_command_message<T: Message>(&self, command: &str, message: T) {
        let command = command.to_string();

        emit(
            self,
            message,
            |message| MessageWrapper { command, message },
            Stream::Stdout,
        );
    }

    fn print_command_error<T: Message>(&self, command: &str, message: T) {
        let command = command.to_string();

        emit(
            self,
            message,
            |error| ErrorWrapper { command, error },
            Stream::Stderr,
        );
    }

    fn print_notification<T: Message>(&self, message: T) {
        emit(
            self,
            message,
            |notification| NotificationWrapper { notification },
            Stream::Stdout,
        );
    }

    #[expect(clippy::disallowed_types)]
    fn print_warning<T: Message>(&self, message: T) {
        use foundry_ui::components::warning::WarningMessage;
        emit(
            self,
            message,
            |warning| WarningWrapper {
                warning: WarningMessage::new(warning),
            },
            Stream::Stdout,
        );
    }

    #[expect(clippy::disallowed_types)]
    fn print_error<T: Message>(&self, message: T) {
        use foundry_ui::components::error::ErrorMessage;
        emit(
            self,
            message,
            |warning| WarningWrapper {
                warning: ErrorMessage::new(warning),
            },
            Stream::Stderr,
        );
    }
}
