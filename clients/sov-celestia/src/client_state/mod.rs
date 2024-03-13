//! Implements the core [`ClientState`](ibc_core::client::context::client_state::ClientState) trait
//! for the Sovereign light client.

mod common;
mod execution;
mod validation;

pub use execution::*;
use ibc_core::client::types::error::ClientError;
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::proto::{Any, Protobuf};
use sov_celestia_client_types::client_state::{
    SovTmClientState, SOV_TENDERMINT_CLIENT_STATE_TYPE_URL,
};
use sov_celestia_client_types::proto::v1::ClientState as RawSovTmClientState;

/// Newtype wrapper exists so that we can bypass Rust's orphan rules and
/// implement traits from `ibc::core::client::context` on the `ClientState`
/// type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, derive_more::From)]
pub struct ClientState(SovTmClientState);

impl ClientState {
    pub fn inner(&self) -> &SovTmClientState {
        &self.0
    }
}

impl Protobuf<RawSovTmClientState> for ClientState {}

impl TryFrom<RawSovTmClientState> for ClientState {
    type Error = ClientError;

    fn try_from(raw: RawSovTmClientState) -> Result<Self, Self::Error> {
        let sov_client_state = SovTmClientState::try_from(raw)?;

        Ok(Self(sov_client_state))
    }
}

impl From<ClientState> for RawSovTmClientState {
    fn from(client_state: ClientState) -> Self {
        client_state.0.into()
    }
}

impl Protobuf<Any> for ClientState {}

impl TryFrom<Any> for ClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        let any = SovTmClientState::try_from(raw)?;

        Ok(Self(any))
    }
}

impl From<ClientState> for Any {
    fn from(client_state: ClientState) -> Self {
        Any {
            type_url: SOV_TENDERMINT_CLIENT_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawSovTmClientState>::encode_vec(client_state),
        }
    }
}
