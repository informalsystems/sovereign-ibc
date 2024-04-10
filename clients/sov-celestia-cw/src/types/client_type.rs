use ibc_core::client::context::client_state::ClientStateExecution;
use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core::client::types::error::ClientError;
use ibc_core::derive::ConsensusState as ConsensusStateDerive;
use ibc_core::primitives::proto::Any;
use sov_celestia_client::client_state::ClientState;
use sov_celestia_client::consensus_state::ConsensusState;
use sov_celestia_client::types::consensus_state::{
    SovTmConsensusState, SOV_TENDERMINT_CONSENSUS_STATE_TYPE_URL,
};

use crate::context::Context;

pub struct SovTmClient;

impl<'a> ClientType<'a> for SovTmClient {
    type ClientState = ClientState;
    type ConsensusState = AnyConsensusState;
}

/// Enables the introduction of custom client and consensus state types tailored
/// for Sovereign light clients.
pub trait ClientType<'a>: Sized {
    type ClientState: ClientStateExecution<Context<'a, Self>> + Clone;
    type ConsensusState: ConsensusStateTrait + Into<Any> + TryFrom<Any, Error = ClientError>;
}

#[derive(Clone, Debug, ConsensusStateDerive)]
pub enum AnyConsensusState {
    Sovereign(ConsensusState),
}

impl From<SovTmConsensusState> for AnyConsensusState {
    fn from(value: SovTmConsensusState) -> Self {
        AnyConsensusState::Sovereign(value.into())
    }
}

impl TryFrom<AnyConsensusState> for SovTmConsensusState {
    type Error = ClientError;

    fn try_from(value: AnyConsensusState) -> Result<Self, Self::Error> {
        match value {
            AnyConsensusState::Sovereign(state) => Ok(state.into_inner()),
        }
    }
}

impl From<AnyConsensusState> for Any {
    fn from(value: AnyConsensusState) -> Self {
        match value {
            AnyConsensusState::Sovereign(cs) => cs.into(),
        }
    }
}

impl TryFrom<Any> for AnyConsensusState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            SOV_TENDERMINT_CONSENSUS_STATE_TYPE_URL => {
                let cs = ConsensusState::try_from(raw)?;
                Ok(AnyConsensusState::Sovereign(cs))
            }
            _ => Err(ClientError::UnknownConsensusStateType {
                consensus_state_type: raw.type_url,
            }),
        }
    }
}
