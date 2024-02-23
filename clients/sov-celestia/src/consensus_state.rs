use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core::client::types::error::ClientError;
use ibc_core::commitment_types::commitment::CommitmentRoot;
use ibc_core::primitives::proto::{Any, Protobuf};
use ibc_core::primitives::Timestamp;
use sov_celestia_client_types::consensus_state::SovTmConsensusState;
use sov_celestia_client_types::proto::v1::ConsensusState as RawConsensusState;
use tendermint::{Hash, Time};

/// Newtype wrapper around the `ConsensusState` type imported from the
/// `ibc-client-tendermint-types` crate. This wrapper exists so that we can
/// bypass Rust's orphan rules and implement traits from
/// `ibc::core::client::context` on the `ConsensusState` type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, derive_more::From)]
pub struct ConsensusState(SovTmConsensusState);

impl ConsensusState {
    pub fn inner(&self) -> &SovTmConsensusState {
        &self.0
    }

    pub fn timestamp(&self) -> Time {
        self.0.da_params.timestamp
    }

    pub fn next_validators_hash(&self) -> Hash {
        self.0.da_params.next_validators_hash
    }
}

impl Protobuf<RawConsensusState> for ConsensusState {}

impl TryFrom<RawConsensusState> for ConsensusState {
    type Error = ClientError;

    fn try_from(raw: RawConsensusState) -> Result<Self, Self::Error> {
        Ok(Self(SovTmConsensusState::try_from(raw)?))
    }
}

impl From<ConsensusState> for RawConsensusState {
    fn from(client_state: ConsensusState) -> Self {
        client_state.0.into()
    }
}

impl Protobuf<Any> for ConsensusState {}

impl TryFrom<Any> for ConsensusState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        Ok(Self(SovTmConsensusState::try_from(raw)?))
    }
}

impl From<ConsensusState> for Any {
    fn from(client_state: ConsensusState) -> Self {
        client_state.0.into()
    }
}

impl ConsensusStateTrait for ConsensusState {
    fn root(&self) -> &CommitmentRoot {
        &self.0.root
    }

    fn timestamp(&self) -> Timestamp {
        let time = self.0.da_params.timestamp.unix_timestamp_nanos();
        Timestamp::from_nanoseconds(time as u64).expect("invalid timestamp")
    }

    fn encode_vec(self) -> Vec<u8> {
        <Self as Protobuf<Any>>::encode_vec(self)
    }
}
