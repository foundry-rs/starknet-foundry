use foundry_ui::Message;
use serde::Serialize;
use serde_json::Value;

pub struct SncastMessage<T: SncastCommandMessage + Serialize>(pub T);

pub trait SncastCommandMessage: Serialize {
    fn text(&self) -> String;
    fn json(&self) -> Value {
        serde_json::to_value(self).expect("Should be serializable to JSON")
    }
}

impl<T> Message for SncastMessage<T>
where
    T: SncastCommandMessage + Serialize,
{
    fn text(&self) -> String {
        self.0.text()
    }

    fn json(&self) -> Value {
        self.0.json()
    }
}
