use bytes::BytesMut;

use crate::error::ConvertBytesToBgpMessageError;

use super::open::OpenMessage;

pub enum Message {
    Open(OpenMessage),
}

impl TryFrom<BytesMut> for Message {
    type Error = ConvertBytesToBgpMessageError;

    fn try_from(value: BytesMut) -> Result<Self, Self::Error> {
        todo!();
    }
}

impl From<Message> for BytesMut {
    fn from(value: Message) -> Self {
        todo!();
    }
}
