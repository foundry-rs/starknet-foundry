use foundry_ui::{Message, OutputFormat, UI as BaseUI};
use serde::Serialize;
use serde_json::Value;

#[derive(Default)]
pub struct UI {
    ui: BaseUI,
}

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

impl UI {
    #[must_use]
    pub fn new(output_format: OutputFormat) -> Self {
        let base_ui = BaseUI::new(output_format);
        Self { ui: base_ui }
    }

    fn should_skip_empty_json(&self, json: &Value) -> bool {
        self.ui.output_format() == OutputFormat::Json && *json == Value::Null
    }

    pub fn print_message<T>(&self, command: &str, message: T)
    where
        T: Message,
    {
        // TODO(#3960) Add better handling for no JSON output
        if self.should_skip_empty_json(&message.json()) {
            return;
        }

        let internal_message = MessageWrapper {
            command: command.to_string(),
            message,
        };
        self.ui.println(&internal_message);
    }

    pub fn print_error<T>(&self, command: &str, message: T)
    where
        T: Message,
    {
        // TODO(#3960) Add better handling for no JSON output
        if self.should_skip_empty_json(&message.json()) {
            return;
        }

        let internal_message = ErrorWrapper {
            command: command.to_string(),
            error: message,
        };
        self.ui.eprintln(&internal_message);
    }

    pub fn print_notification<T>(&self, message: T)
    where
        T: Message,
    {
        // TODO(#3960) Add better handling for no JSON output
        if self.should_skip_empty_json(&message.json()) {
            return;
        }

        let internal_message = NotificationWrapper {
            notification: message,
        };
        self.ui.println(&internal_message);
    }

    pub fn print_warning<T>(&self, message: T)
    where
        T: Message,
    {
        // TODO(#3960) Add better handling for no JSON output
        if self.should_skip_empty_json(&message.json()) {
            return;
        }

        let internal_message = WarningWrapper { warning: message };
        self.ui.println(&internal_message);
    }

    pub fn print_blank_line(&self) {
        self.ui.print_blank_line();
    }

    #[must_use]
    pub fn base_ui(&self) -> &BaseUI {
        &self.ui
    }
}
