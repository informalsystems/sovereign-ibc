pub mod context;

use derive_more::{From, TryInto};
use ibc_client_tendermint::client_state::ClientState as TmClientState;
use ibc_client_tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc_client_tendermint::types::{
    ClientState as TmClientStateTypes, ConsensusState as TmConsensusStateType,
    TENDERMINT_CLIENT_STATE_TYPE_URL, TENDERMINT_CONSENSUS_STATE_TYPE_URL,
};
use ibc_core::client::types::error::ClientError;
use ibc_core::derive::{ClientState, ConsensusState};
use ibc_core::primitives::proto::{Any, Protobuf};
use sov_celestia_client::client_state::ClientState as SovClientState;
use sov_celestia_client::consensus_state::ConsensusState as SovConsensusState;
use sov_celestia_client::types::client_state::{
    SovTmClientState, SOV_TENDERMINT_CLIENT_STATE_TYPE_URL,
};
use sov_celestia_client::types::consensus_state::{
    SovTmConsensusState, SOV_TENDERMINT_CONSENSUS_STATE_TYPE_URL,
};
use sov_modules_api::{Spec, TxState};

use crate::context::IbcContext;

#[derive(Clone, Debug, From, TryInto, ClientState)]
#[validation(IbcContext<'a, S: Spec, TS: TxState<S>>)]
#[execution(IbcContext<'a, S: Spec, TS: TxState<S>>)]
pub enum AnyClientState {
    Tendermint(TmClientState),
    Sovereign(SovClientState),
}

impl From<SovTmClientState> for AnyClientState {
    fn from(cs: SovTmClientState) -> Self {
        Self::Sovereign(cs.into())
    }
}

impl From<TmClientStateTypes> for AnyClientState {
    fn from(cs: TmClientStateTypes) -> Self {
        Self::Tendermint(cs.into())
    }
}

impl Protobuf<Any> for AnyClientState {}

impl TryFrom<Any> for AnyClientState {
    type Error = ClientError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            TENDERMINT_CLIENT_STATE_TYPE_URL => {
                let tm_cs: TmClientState = value.try_into()?;
                Ok(Self::Tendermint(tm_cs))
            }
            SOV_TENDERMINT_CLIENT_STATE_TYPE_URL => {
                let sov_cs: SovClientState = value.try_into()?;
                Ok(Self::Sovereign(sov_cs))
            }
            _ => Err(ClientError::UnknownClientStateType {
                client_state_type: value.type_url,
            }),
        }
    }
}

impl From<AnyClientState> for Any {
    fn from(any_cs: AnyClientState) -> Self {
        match any_cs {
            AnyClientState::Tendermint(tm_cs) => tm_cs.into(),
            AnyClientState::Sovereign(sov_cs) => sov_cs.into(),
        }
    }
}

#[derive(Clone, From, TryInto, ConsensusState)]
pub enum AnyConsensusState {
    Tendermint(TmConsensusState),
    Sovereign(SovConsensusState),
}

impl TryFrom<AnyConsensusState> for TmConsensusStateType {
    type Error = ClientError;

    fn try_from(cs: AnyConsensusState) -> Result<TmConsensusStateType, Self::Error> {
        match cs {
            AnyConsensusState::Tendermint(cs) => Ok(cs.into_inner()),
            AnyConsensusState::Sovereign(_) => Err(ClientError::UnknownConsensusStateType {
                consensus_state_type: "sovereign".to_string(),
            }),
        }
    }
}

impl TryFrom<AnyConsensusState> for SovTmConsensusState {
    type Error = ClientError;

    fn try_from(cs: AnyConsensusState) -> Result<SovTmConsensusState, Self::Error> {
        match cs {
            AnyConsensusState::Tendermint(_) => Err(ClientError::UnknownConsensusStateType {
                consensus_state_type: "tendermint".to_string(),
            }),
            AnyConsensusState::Sovereign(cs) => Ok(cs.into_inner()),
        }
    }
}

impl From<SovTmConsensusState> for AnyConsensusState {
    fn from(cs: SovTmConsensusState) -> Self {
        Self::Sovereign(cs.into())
    }
}

impl From<TmConsensusStateType> for AnyConsensusState {
    fn from(cs: TmConsensusStateType) -> Self {
        Self::Tendermint(cs.into())
    }
}

impl Protobuf<Any> for AnyConsensusState {}

impl TryFrom<Any> for AnyConsensusState {
    type Error = ClientError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            TENDERMINT_CONSENSUS_STATE_TYPE_URL => {
                let tm_cs: TmConsensusState = value.try_into()?;
                Ok(Self::Tendermint(tm_cs))
            }
            SOV_TENDERMINT_CONSENSUS_STATE_TYPE_URL => {
                let sov_cs: SovConsensusState = value.try_into()?;
                Ok(Self::Sovereign(sov_cs))
            }
            _ => Err(ClientError::UnknownConsensusStateType {
                consensus_state_type: value.type_url,
            }),
        }
    }
}

impl From<AnyConsensusState> for Any {
    fn from(any_cs: AnyConsensusState) -> Self {
        match any_cs {
            AnyConsensusState::Tendermint(tm_cs) => tm_cs.into(),
            AnyConsensusState::Sovereign(sov_cs) => sov_cs.into(),
        }
    }
}
