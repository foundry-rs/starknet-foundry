use foundry_ui::Message;

use crate::NumbersFormat;

pub struct CastMessage<T: Message> {
    pub numbers_format: NumbersFormat,
    pub message: T,
}
