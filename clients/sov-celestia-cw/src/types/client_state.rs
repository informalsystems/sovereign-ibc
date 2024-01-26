use ibc_core::client::types::error::ClientError;
use ibc_core::derive::ClientState as ClientStateDerive;
use ibc_core::primitives::proto::{Any, Protobuf};
use sov_celestia_client::client_state::ClientState;
use sov_celestia_client::types::client_state::{
    ClientState as ClientStateType, SOV_TENDERMINT_CLIENT_STATE_TYPE_URL,
};

use crate::contexts::Context;

#[derive(ClientStateDerive, Debug, derive_more::From)]
#[validation(Context<'a>)]
#[execution(Context<'a>)]
pub enum AnyClientState {
    Sovereign(ClientState),
}

impl TryFrom<AnyClientState> for ClientState {
    type Error = ClientError;

    fn try_from(value: AnyClientState) -> Result<Self, Self::Error> {
        match value {
            AnyClientState::Sovereign(state) => Ok(state),
        }
    }
}

impl From<ClientStateType> for AnyClientState {
    fn from(value: ClientStateType) -> Self {
        AnyClientState::Sovereign(ClientState::from(value))
    }
}

impl From<AnyClientState> for Any {
    fn from(value: AnyClientState) -> Self {
        match value {
            AnyClientState::Sovereign(cs) => Any {
                type_url: SOV_TENDERMINT_CLIENT_STATE_TYPE_URL.to_string(),
                value: Protobuf::<Any>::encode_vec(cs),
            },
        }
    }
}
