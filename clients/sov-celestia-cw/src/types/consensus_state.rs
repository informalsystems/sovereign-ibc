use ibc_client_tendermint::consensus_state::ConsensusState;
use ibc_client_tendermint::types::TENDERMINT_CONSENSUS_STATE_TYPE_URL;
use ibc_core::client::types::error::ClientError;
use ibc_core::derive::ConsensusState;
use ibc_core::primitives::proto::{Any, Protobuf};

#[derive(Clone, Debug, derive_more::From, ConsensusState)]
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
                type_url: TENDERMINT_CONSENSUS_STATE_TYPE_URL.to_string(),
                value: Protobuf::<Any>::encode_vec(cs),
            },
        }
    }
}
