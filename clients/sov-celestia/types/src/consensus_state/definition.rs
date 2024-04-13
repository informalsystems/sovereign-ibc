//! Defines the `ConsensusState` type for the Sovereign SDK rollups

use ibc_core::client::types::error::ClientError;
use ibc_core::commitment_types::commitment::CommitmentRoot;
use ibc_core::primitives::proto::{Any, Protobuf};
use tendermint::Time;

use super::TmConsensusParams;
use crate::client_message::SovTmHeader;
use crate::proto::v1::ConsensusState as RawConsensusState;
use crate::sovereign::{Error, SovereignConsensusParams};

pub const SOV_TENDERMINT_CONSENSUS_STATE_TYPE_URL: &str =
    "/ibc.lightclients.sovereign.tendermint.v1.ConsensusState";

/// Defines the generic `ConsensusState` type for the Sovereign SDK rollups
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsensusState<Da> {
    pub sovereign_params: SovereignConsensusParams,
    pub da_params: Da,
}

impl<Da> ConsensusState<Da> {
    pub fn new(sovereign_params: SovereignConsensusParams, da_params: Da) -> Self {
        Self {
            sovereign_params,
            da_params,
        }
    }
}

pub type SovTmConsensusState = ConsensusState<TmConsensusParams>;

impl SovTmConsensusState {
    /// Returns the timestamp of the consensus state
    pub fn timestamp(&self) -> Time {
        self.da_params.timestamp
    }
}

impl Protobuf<RawConsensusState> for SovTmConsensusState {}

impl TryFrom<RawConsensusState> for SovTmConsensusState {
    type Error = ClientError;

    fn try_from(raw: RawConsensusState) -> Result<Self, Self::Error> {
        Ok(Self::new(
            raw.sovereign_params
                .ok_or(Error::missing("sovereign_params"))?
                .try_into()?,
            raw.tendermint_params
                .ok_or(Error::missing("tendermint_params"))?
                .try_into()?,
        ))
    }
}

impl From<SovTmConsensusState> for RawConsensusState {
    fn from(value: SovTmConsensusState) -> Self {
        RawConsensusState {
            sovereign_params: Some(value.sovereign_params.into()),
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
            sovereign_params: CommitmentRoot::from_bytes(header.app_hash.as_ref()).into(),
            da_params: TmConsensusParams::new(header.time, header.next_validators_hash),
        }
    }
}

impl From<SovTmHeader> for SovTmConsensusState {
    fn from(header: SovTmHeader) -> Self {
        let tm_header = header.da_header.signed_header.header;

        Self::new(
            CommitmentRoot::from_bytes(header.aggregated_proof.final_state_root().as_ref()).into(),
            TmConsensusParams::new(tm_header.time, tm_header.next_validators_hash),
        )
    }
}
