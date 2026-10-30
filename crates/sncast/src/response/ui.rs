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

fn should_skip_empty_json(format: OutputFormat, json: &Value) -> bool {
    format == OutputFormat::Json && *json == Value::Null
}

pub trait UIExt {
    fn print_message<T: Message>(&self, command: &str, message: T);
    fn print_error<T: Message>(&self, command: &str, message: T);
    fn print_notification<T: Message>(&self, message: T);
    fn print_warning<T: Message>(&self, message: T);
}

impl UIExt for UI {
    fn print_message<T: Message>(&self, command: &str, message: T) {
        // TODO(#3960) Add better handling for no JSON output
        if should_skip_empty_json(self.output_format(), &message.json()) {
            return;
        }

        let internal_message = MessageWrapper {
            command: command.to_string(),
            message,
        };
        self.println(&internal_message);
    }

    fn print_error<T: Message>(&self, command: &str, message: T) {
        // TODO(#3960) Add better handling for no JSON output
        if should_skip_empty_json(self.output_format(), &message.json()) {
            return;
        }

        let internal_message = ErrorWrapper {
            command: command.to_string(),
            error: message,
        };
        self.eprintln(&internal_message);
    }

    fn print_notification<T: Message>(&self, message: T) {
        // TODO(#3960) Add better handling for no JSON output
        if should_skip_empty_json(self.output_format(), &message.json()) {
            return;
        }

        let internal_message = NotificationWrapper {
            notification: message,
        };
        self.println(&internal_message);
    }

    fn print_warning<T: Message>(&self, message: T) {
        // TODO(#3960) Add better handling for no JSON output
        if should_skip_empty_json(self.output_format(), &message.json()) {
            return;
        }

        let internal_message = WarningWrapper { warning: message };
        self.println(&internal_message);
    }
}
