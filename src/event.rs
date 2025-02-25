use crate::packets::{keepalive::KeepaliveMessage, open::OpenMessage, update::UpdateMessage};

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Event {
    ManualStart,
    TcpConnectionConfirmed,
    BgpOpen(OpenMessage),
    KeepaliveMsg(KeepaliveMessage),
    UpdateMsg(UpdateMessage),
    Established,
}
