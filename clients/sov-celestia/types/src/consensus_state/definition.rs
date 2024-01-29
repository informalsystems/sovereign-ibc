//! Defines Sovereign's `ConsensusState` type

use ibc_core::client::types::error::ClientError;
use ibc_core::commitment_types::commitment::CommitmentRoot;
use ibc_core::primitives::proto::{Any, Protobuf};
use ibc_proto::ibc::lightclients::sovereign::tendermint::v1::ConsensusState as RawConsensusState;
use tendermint::hash::Algorithm;
use tendermint::time::Time;
use tendermint::Hash;
use tendermint_proto::google::protobuf as tpb;

use crate::client_message::SovTmHeader;

pub const SOV_TENDERMINT_CONSENSUS_STATE_TYPE_URL: &str =
    "/ibc.lightclients.sovereign.tendermint.v1.ConsensusState";

/// Defines the Sovereign light client's consensus state
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsensusState {
    pub timestamp: Time,
    pub root: CommitmentRoot,
    pub next_validators_hash: Hash,
}

impl ConsensusState {
    pub fn new(root: CommitmentRoot, timestamp: Time, next_validators_hash: Hash) -> Self {
        Self {
            timestamp,
            root,
            next_validators_hash,
        }
    }
}

impl Protobuf<RawConsensusState> for ConsensusState {}

impl TryFrom<RawConsensusState> for ConsensusState {
    type Error = ClientError;

    fn try_from(raw: RawConsensusState) -> Result<Self, Self::Error> {
        let proto_root = raw
            .root
            .ok_or(ClientError::Other {
                description: "missing root".to_string(),
            })?
            .hash;

        let ibc_proto::google::protobuf::Timestamp { seconds, nanos } =
            raw.timestamp.ok_or(ClientError::Other {
                description: "missing timestamp".to_string(),
            })?;

        let proto_timestamp = tpb::Timestamp { seconds, nanos };
        let timestamp = proto_timestamp.try_into().map_err(|_| ClientError::Other {
            description: "invalid timestamp".to_string(),
        })?;

        let next_validators_hash = Hash::from_bytes(Algorithm::Sha256, &raw.next_validators_hash)
            .map_err(|_| ClientError::Other {
            description: "invalid next validators hash".to_string(),
        })?;

        Ok(Self {
            root: proto_root.into(),
            timestamp,
            next_validators_hash,
        })
    }
}

impl From<ConsensusState> for RawConsensusState {
    fn from(value: ConsensusState) -> Self {
        let tpb::Timestamp { seconds, nanos } = value.timestamp.into();
        let timestamp = ibc_proto::google::protobuf::Timestamp { seconds, nanos };

        RawConsensusState {
            timestamp: Some(timestamp),
            root: Some(ibc_proto::ibc::core::commitment::v1::MerkleRoot {
                hash: value.root.into_vec(),
            }),
            next_validators_hash: value.next_validators_hash.as_bytes().to_vec(),
        }
    }
}

impl Protobuf<Any> for ConsensusState {}

impl TryFrom<Any> for ConsensusState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        fn decode_consensus_state(value: &[u8]) -> Result<ConsensusState, ClientError> {
            let consensus_state =
                Protobuf::<RawConsensusState>::decode(value).map_err(|e| ClientError::Other {
                    description: e.to_string(),
                })?;
            Ok(consensus_state)
        }

        match raw.type_url.as_str() {
            SOV_TENDERMINT_CONSENSUS_STATE_TYPE_URL => decode_consensus_state(&raw.value),
            _ => Err(ClientError::UnknownConsensusStateType {
                consensus_state_type: raw.type_url,
            }),
        }
    }
}

impl From<ConsensusState> for Any {
    fn from(consensus_state: ConsensusState) -> Self {
        Any {
            type_url: SOV_TENDERMINT_CONSENSUS_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawConsensusState>::encode_vec(consensus_state),
        }
    }
}

impl From<tendermint::block::Header> for ConsensusState {
    fn from(header: tendermint::block::Header) -> Self {
        Self {
            root: CommitmentRoot::from_bytes(header.app_hash.as_ref()),
            timestamp: header.time,
            next_validators_hash: header.next_validators_hash,
        }
    }
}

impl From<SovTmHeader> for ConsensusState {
    fn from(header: SovTmHeader) -> Self {
        let tm_header = header.da_header.signed_header.header;

        Self {
            root: CommitmentRoot::from_bytes(
                header
                    .aggregated_proof_data
                    .public_input
                    .final_state_root
                    .as_ref(),
            ),
            timestamp: tm_header.time,
            next_validators_hash: tm_header.next_validators_hash,
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_util {
    use ibc_client_wasm_types::consensus_state::ConsensusState as WasmConsensusState;

    use super::*;

    impl ConsensusState {
        pub fn into_wasm(&self) -> WasmConsensusState {
            WasmConsensusState {
                data: Any::from(self.clone()).value,
            }
        }
    }
}