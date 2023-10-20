use alloc::string::ToString;

#[cfg(feature = "cosmwasm")]
use cosmwasm_schema::cw_serde;
use ibc::core::ics02_client::error::ClientError;
use ibc::proto::protobuf::Protobuf;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::wasm::v1::ConsensusState as RawConsensusState;

#[cfg(feature = "cosmwasm")]
use crate::serializer::Base64;
use crate::Bytes;

pub const WASM_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.ConsensusState";

#[cfg_attr(feature = "cosmwasm", cw_serde)]
#[cfg_attr(not(feature = "cosmwasm"), derive(Clone, Debug, PartialEq))]
#[derive(Eq)]
pub struct ConsensusState {
    #[cfg_attr(feature = "cosmwasm", schemars(with = "String"))]
    #[cfg_attr(feature = "cosmwasm", serde(with = "Base64", default))]
    pub data: Bytes,
}

impl ConsensusState {
    pub fn new(data: Bytes) -> Self {
        Self { data }
    }
}

impl Protobuf<RawConsensusState> for ConsensusState {}

impl From<ConsensusState> for RawConsensusState {
    fn from(value: ConsensusState) -> Self {
        Self { data: value.data }
    }
}

impl TryFrom<RawConsensusState> for ConsensusState {
    type Error = ClientError;

    fn try_from(value: RawConsensusState) -> Result<Self, Self::Error> {
        Ok(Self { data: value.data })
    }
}

impl Protobuf<Any> for ConsensusState {}

impl From<ConsensusState> for Any {
    fn from(value: ConsensusState) -> Self {
        Self {
            type_url: WASM_CONSENSUS_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawConsensusState>::encode_vec(&value),
        }
    }
}

impl TryFrom<Any> for ConsensusState {
    type Error = ClientError;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        use core::ops::Deref;

        use bytes::Buf;
        use prost::Message;

        fn decode_consensus_state<B: Buf>(buf: B) -> Result<ConsensusState, ClientError> {
            RawConsensusState::decode(buf)
                .map_err(ClientError::Decode)?
                .try_into()
        }
        match any.type_url.as_str() {
            WASM_CONSENSUS_STATE_TYPE_URL => {
                decode_consensus_state(any.value.deref()).map_err(Into::into)
            }
            _ => Err(ClientError::Other {
                description: "type_url does not match".into(),
            }),
        }
    }
}
