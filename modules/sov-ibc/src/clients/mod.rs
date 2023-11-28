pub mod context;

use derive_more::{From, TryInto};
use ibc_client_tendermint::client_state::ClientState as TmClientState;
use ibc_client_tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc_core::client::context::consensus_state::ConsensusState;
use ibc_core::client::types::error::ClientError;
use ibc_core::commitment_types::commitment;
use ibc_core::primitives;
use ibc_core::primitives::proto::{Any, Protobuf};

#[derive(Clone, From, TryInto)]
pub enum AnyConsensusState {
    Tendermint(TmConsensusState),
}

impl TryFrom<Any> for AnyConsensusState {
    type Error = ClientError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        let tm_cs: TmConsensusState = value.try_into()?;

        Ok(Self::Tendermint(tm_cs))
    }
}

impl From<AnyConsensusState> for Any {
    fn from(any_cs: AnyConsensusState) -> Self {
        match any_cs {
            AnyConsensusState::Tendermint(tm_cs) => tm_cs.into(),
        }
    }
}

impl Protobuf<Any> for AnyConsensusState {}

#[derive(Clone, From, TryInto)]
pub enum AnyClientState {
    Tendermint(TmClientState),
}

impl TryFrom<Any> for AnyClientState {
    type Error = ClientError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        let tm_cs: TmClientState = value.try_into()?;

        Ok(Self::Tendermint(tm_cs))
    }
}

impl From<AnyClientState> for Any {
    fn from(any_cs: AnyClientState) -> Self {
        match any_cs {
            AnyClientState::Tendermint(tm_cs) => tm_cs.into(),
        }
    }
}

impl Protobuf<Any> for AnyClientState {}

impl ConsensusState for AnyConsensusState {
    fn root(&self) -> &commitment::CommitmentRoot {
        match self {
            AnyConsensusState::Tendermint(cs) => cs.root(),
        }
    }

    fn timestamp(&self) -> primitives::Timestamp {
        match self {
            AnyConsensusState::Tendermint(cs) => cs.timestamp().into(),
        }
    }

    fn encode_vec(self) -> Vec<u8> {
        match self {
            AnyConsensusState::Tendermint(cs) => {
                <TmConsensusState as ConsensusState>::encode_vec(cs)
            }
        }
    }
}
