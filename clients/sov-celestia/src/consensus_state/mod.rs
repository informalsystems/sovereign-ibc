pub mod definition;
use alloc::vec::Vec;

pub use definition::ConsensusState as SovConsensusState;
use ibc_core::client::context::consensus_state::ConsensusState;
use ibc_core::commitment_types::commitment::CommitmentRoot;
use ibc_core::primitives::proto::Any;
use ibc_core::primitives::Timestamp;
use tendermint_proto::Protobuf;

use super::error::Error;

#[derive(Clone, Debug)]
pub enum AnyConsensusState {
    Sovereign(SovConsensusState),
}

impl From<SovConsensusState> for AnyConsensusState {
    fn from(value: SovConsensusState) -> Self {
        Self::Sovereign(value)
    }
}

impl TryFrom<AnyConsensusState> for SovConsensusState {
    type Error = Error;

    fn try_from(value: AnyConsensusState) -> Result<Self, Self::Error> {
        match value {
            AnyConsensusState::Sovereign(state) => Ok(state),
        }
    }
}

impl From<AnyConsensusState> for Any {
    fn from(value: AnyConsensusState) -> Self {
        match value {
            AnyConsensusState::Sovereign(cs) => Any {
                type_url: definition::SOVEREIGN_CONSENSUS_STATE_TYPE_URL.to_string(),
                value: Protobuf::<Any>::encode_vec(&cs).unwrap(),
            },
        }
    }
}

impl ConsensusState for AnyConsensusState {
    fn root(&self) -> &CommitmentRoot {
        todo!()
    }

    fn timestamp(&self) -> Timestamp {
        todo!()
    }

    fn encode_vec(self) -> Vec<u8> {
        todo!()
    }
}
