use alloc::string::ToString;

#[cfg(feature = "cosmwasm")]
use cosmwasm_schema::cw_serde;
use ibc::core::ics02_client::error::ClientError;
use ibc::proto::protobuf::Protobuf;
use ibc::Height;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::wasm::v1::ClientState as RawClientState;

#[cfg(feature = "cosmwasm")]
use crate::serializer::Base64;
use crate::Bytes;

pub const WASM_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.ClientState";

#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Clone, Debug, PartialEq))]
#[derive(Eq)]
pub struct ClientState {
    #[cfg_attr(feature = "cosmwasm", schemars(with = "String"))]
    #[cfg_attr(feature = "cosmwasm", serde(with = "Base64", default))]
    pub data: Bytes,
    #[cfg_attr(feature = "cosmwasm", schemars(with = "String"))]
    #[cfg_attr(feature = "cosmwasm", serde(with = "Base64", default))]
    pub code_hash: Bytes,
    pub latest_height: Height,
}

impl Protobuf<RawClientState> for ClientState {}

impl From<ClientState> for RawClientState {
    fn from(value: ClientState) -> Self {
        Self {
            data: value.data,
            code_hash: value.code_hash,
            latest_height: Some(value.latest_height.into()),
        }
    }
}

impl TryFrom<RawClientState> for ClientState {
    type Error = ClientError;

    fn try_from(raw: RawClientState) -> Result<Self, Self::Error> {
        let latest_height = raw
            .latest_height
            .ok_or(ClientError::Other {
                description: "missing latest_height".to_string(),
            })?
            .try_into()
            .map_err(|_| ClientError::Other {
                description: "invalid latest_height".to_string(),
            })?;
        Ok(Self {
            data: raw.data,
            code_hash: raw.code_hash,
            latest_height,
        })
    }
}

impl Protobuf<Any> for ClientState {}

impl From<ClientState> for Any {
    fn from(value: ClientState) -> Self {
        Self {
            type_url: WASM_CLIENT_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawClientState>::encode_vec(&value),
        }
    }
}

impl TryFrom<Any> for ClientState {
    type Error = ClientError;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        use core::ops::Deref;

        use bytes::Buf;
        use prost::Message;

        fn decode_client_state<B: Buf>(buf: B) -> Result<ClientState, ClientError> {
            RawClientState::decode(buf)
                .map_err(ClientError::Decode)?
                .try_into()
        }

        match any.type_url.as_str() {
            WASM_CLIENT_STATE_TYPE_URL => {
                decode_client_state(any.value.deref()).map_err(Into::into)
            }
            _ => Err(ClientError::Other {
                description: "type_url does not match".into(),
            }),
        }
    }
}
