use ibc_core::client::types::error::ClientError;
use ibc_core::derive::ConsensusState as ConsensusStateDerive;
use ibc_core::primitives::proto::{Any, Protobuf};
use sov_celestia_client::consensus_state::ConsensusState;
use sov_celestia_client::types::consensus_state::SOV_TENDERMINT_CONSENSUS_STATE_TYPE_URL;

#[derive(Clone, Debug, derive_more::From, ConsensusStateDerive)]
pub enum AnyConsensusState {
    Sovereign(ConsensusState),
}

impl TryFrom<AnyConsensusState> for ConsensusState {
    type Error = ClientError;

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
                type_url: SOV_TENDERMINT_CONSENSUS_STATE_TYPE_URL.to_string(),
                value: Protobuf::<Any>::encode_vec(cs),
            },
        }
    }
}
