use foundry_ui::{Message, formats::NumbersFormat};

pub struct CastMessage<T: Message> {
    pub numbers_format: NumbersFormat,
    pub message: T,
}
