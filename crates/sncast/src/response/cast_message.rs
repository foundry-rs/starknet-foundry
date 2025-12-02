use foundry_ui::Message;
use serde::Serialize;
use serde_json::Value;

pub struct SncastMessage<T: SncastCommandMessage + Serialize>(pub T);

pub trait SncastCommandMessage {
    fn text(&self) -> String;
}

impl<T> Message for SncastMessage<T>
where
    T: SncastCommandMessage + Serialize,
{
    fn text(&self) -> String {
        self.0.text()
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.0).expect("Should be serializable to JSON")
    }
}
