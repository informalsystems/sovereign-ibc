pub mod context;

use derive_more::{From, TryInto};
use ibc::clients::ics07_tendermint::client_state::ClientState as TmClientState;
use ibc::clients::ics07_tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc::core::ics02_client::consensus_state::ConsensusState;
use ibc::core::ics02_client::error::ClientError;
use ibc::Any;
use ibc_proto::protobuf::Protobuf;

#[derive(Clone, From, TryInto, ConsensusState)]
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
