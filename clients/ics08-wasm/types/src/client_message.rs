use ibc_core_client::types::error::ClientError;
use ibc_primitives::proto::Protobuf;
use ibc_proto::ibc::lightclients::wasm::v1::ClientMessage as RawClientMessage;

use crate::Bytes;

pub const WASM_CLIENT_MESSAGE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.ClientMessage";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientMessage {
    pub data: Bytes,
}

impl Protobuf<RawClientMessage> for ClientMessage {}

impl TryFrom<RawClientMessage> for ClientMessage {
    type Error = ClientError;

    fn try_from(raw: RawClientMessage) -> Result<Self, Self::Error> {
        Ok(Self { data: raw.data })
    }
}

impl From<ClientMessage> for RawClientMessage {
    fn from(value: ClientMessage) -> Self {
        RawClientMessage { data: value.data }
    }
}
