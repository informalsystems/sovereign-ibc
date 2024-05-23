//! Implements the core
//! [`ClientState`](ibc_core::client::context::client_state::ClientState) trait
//! for the Sovereign light client.

mod common;
mod execution;
mod misbehaviour;
mod update_client;
mod validation;

pub use execution::*;
use ibc_client_wasm_types::client_state::ClientState as WasmClientState;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::proto::{Any, Protobuf};
use ibc_proto::ibc::lightclients::wasm::v1::ClientState as RawWasmClientState;
pub use misbehaviour::*;
use prost::Message;
use sov_celestia_client_types::client_state::{
    SovTmClientState, SOV_TENDERMINT_CLIENT_STATE_TYPE_URL,
};
use sov_celestia_client_types::proto::v1::ClientState as RawSovTmClientState;
pub use update_client::*;

/// Newtype wrapper exists so that we can bypass Rust's orphan rules and
/// implement traits from `ibc::core::client::context` on the `ClientState`
/// type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum ClientState {
    Native {
        state: SovTmClientState,
    },
    Wasm {
        state: SovTmClientState,
        checksum: Vec<u8>,
        latest_height: Height,
    },
}

impl ClientState {
    pub fn inner(&self) -> &SovTmClientState {
        match self {
            Self::Native { state } | Self::Wasm { state, .. } => state,
        }
    }

    pub fn into_inner(self) -> SovTmClientState {
        match self {
            Self::Native { state } | Self::Wasm { state, .. } => state,
        }
    }

    pub fn native(state: SovTmClientState) -> Self {
        Self::Native { state }
    }

    pub fn wasm(state: SovTmClientState, checksum: Vec<u8>, latest_height: Height) -> Self {
        Self::Wasm {
            state,
            checksum,
            latest_height,
        }
    }
}

impl From<SovTmClientState> for ClientState {
    fn from(value: SovTmClientState) -> Self {
        Self::native(value)
    }
}

impl Protobuf<RawSovTmClientState> for ClientState {}

impl TryFrom<RawSovTmClientState> for ClientState {
    type Error = ClientError;

    fn try_from(raw: RawSovTmClientState) -> Result<Self, Self::Error> {
        let sov_client_state = SovTmClientState::try_from(raw)?;
        Ok(Self::native(sov_client_state))
    }
}

impl From<ClientState> for RawSovTmClientState {
    fn from(client_state: ClientState) -> Self {
        client_state.into_inner().into()
    }
}

impl TryFrom<WasmClientState> for ClientState {
    type Error = ClientError;

    fn try_from(value: WasmClientState) -> Result<Self, Self::Error> {
        let any_data = Any::decode(value.data.as_slice()).map_err(|err| ClientError::Other {
            description: format!("Expected Any: {err}"),
        })?;

        let state = any_data.try_into()?;

        // note: state.latest_height_in_sov() and value.latest_height should be equal.

        Ok(Self::wasm(state, value.checksum, value.latest_height))
    }
}

impl TryFrom<ClientState> for WasmClientState {
    type Error = ClientError;

    fn try_from(value: ClientState) -> Result<Self, Self::Error> {
        match value {
            ClientState::Wasm {
                state,
                checksum,
                latest_height,
            } => Ok(Self {
                data: Any::from(state).encode_to_vec(),
                checksum,
                latest_height,
            }),
            _ => Err(ClientError::Other {
                description: "Wasm client state expected.".into(),
            }),
        }
    }
}

impl TryFrom<RawWasmClientState> for ClientState {
    type Error = ClientError;

    fn try_from(value: RawWasmClientState) -> Result<Self, Self::Error> {
        let wasm_client = WasmClientState::try_from(value).map_err(|_| ClientError::Other {
            description: "not wasm client state".into(),
        })?;
        wasm_client.try_into()
    }
}

impl TryFrom<ClientState> for RawWasmClientState {
    type Error = ClientError;

    fn try_from(client_state: ClientState) -> Result<Self, Self::Error> {
        let wasm_client = WasmClientState::try_from(client_state)?;
        Ok(wasm_client.into())
    }
}

impl Protobuf<Any> for ClientState {}

impl TryFrom<Any> for ClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        WasmClientState::try_from(raw.clone())
            .map_err(|_| ClientError::Other {
                description: "not wasm client state".into(),
            })
            .and_then(|wasm_state| {
                let any_data =
                    Any::decode(wasm_state.data.as_slice()).map_err(|err| ClientError::Other {
                        description: format!("Expected Any: {err}"),
                    })?;
                Ok(Self::wasm(
                    any_data.try_into()?,
                    wasm_state.checksum,
                    wasm_state.latest_height,
                ))
            })
            .or_else(|_| SovTmClientState::try_from(raw).map(Self::native))
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
