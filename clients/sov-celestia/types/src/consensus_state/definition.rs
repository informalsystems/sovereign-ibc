//! Defines the `ConsensusState` type for the Sovereign SDK rollups

use ibc_core::client::types::error::ClientError;
use ibc_core::commitment_types::commitment::CommitmentRoot;
use ibc_core::primitives::proto::{Any, Protobuf};
use ibc_proto::ibc::lightclients::sovereign::tendermint::v1::ConsensusState as RawConsensusState;

use super::TmConsensusParams;
use crate::client_message::SovTmHeader;

pub const SOV_TENDERMINT_CONSENSUS_STATE_TYPE_URL: &str =
    "/ibc.lightclients.sovereign.tendermint.v1.ConsensusState";

/// Defines the Sovereign light client's consensus state
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsensusState<Da> {
    pub root: CommitmentRoot,
    pub da_params: Da,
}

impl<Da> ConsensusState<Da> {
    pub fn new(root: CommitmentRoot, da_params: Da) -> Self {
        Self { root, da_params }
    }
}
pub type SovTmConsensusState = ConsensusState<TmConsensusParams>;

impl Protobuf<RawConsensusState> for SovTmConsensusState {}

impl TryFrom<RawConsensusState> for SovTmConsensusState {
    type Error = ClientError;

    fn try_from(raw: RawConsensusState) -> Result<Self, Self::Error> {
        let proto_root = raw
            .root
            .ok_or(ClientError::Other {
                description: "missing root".to_string(),
            })?
            .hash;

        let da_params = raw
            .tendermint_params
            .ok_or(ClientError::Other {
                description: "missing tendermint params".to_string(),
            })?
            .try_into()?;

        Ok(Self {
            root: proto_root.into(),
            da_params,
        })
    }
}

impl From<SovTmConsensusState> for RawConsensusState {
    fn from(value: SovTmConsensusState) -> Self {
        RawConsensusState {
            root: Some(ibc_proto::ibc::core::commitment::v1::MerkleRoot {
                hash: value.root.into_vec(),
            }),
            tendermint_params: Some(value.da_params.into()),
        }
    }
}

impl Protobuf<Any> for SovTmConsensusState {}

impl TryFrom<Any> for SovTmConsensusState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        fn decode_consensus_state(value: &[u8]) -> Result<SovTmConsensusState, ClientError> {
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

impl From<SovTmConsensusState> for Any {
    fn from(consensus_state: SovTmConsensusState) -> Self {
        Any {
            type_url: SOV_TENDERMINT_CONSENSUS_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawConsensusState>::encode_vec(consensus_state),
        }
    }
}

impl From<tendermint::block::Header> for SovTmConsensusState {
    fn from(header: tendermint::block::Header) -> Self {
        Self {
            root: CommitmentRoot::from_bytes(header.app_hash.as_ref()),
            da_params: TmConsensusParams {
                timestamp: header.time,
                next_validators_hash: header.next_validators_hash,
            },
        }
    }
}

impl From<SovTmHeader> for SovTmConsensusState {
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
            da_params: TmConsensusParams {
                timestamp: tm_header.time,
                next_validators_hash: tm_header.next_validators_hash,
            },
        }
    }
}
