use ibc_client_wasm_types::consensus_state::ConsensusState as WasmConsensusState;
use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core::client::types::error::ClientError;
use ibc_core::commitment_types::commitment::CommitmentRoot;
use ibc_core::primitives::proto::{Any, Protobuf};
use ibc_core::primitives::Timestamp;
use ibc_proto::ibc::lightclients::wasm::v1::ConsensusState as RawWasmConsensusState;
use prost::Message;
use sov_celestia_client_types::consensus_state::SovTmConsensusState;
use sov_celestia_client_types::proto::v1::ConsensusState as RawConsensusState;
use tendermint::{Hash, Time};

/// Newtype wrapper around the `ConsensusState` type imported from the
/// `sov-client-celestia-types` crate. This wrapper exists so that we can bypass
/// Rust's orphan rules and implement traits from `ibc::core::client::context`
/// on the `ConsensusState` type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum ConsensusState {
    Native { state: SovTmConsensusState },
    Wasm { state: SovTmConsensusState },
}

impl ConsensusState {
    pub fn inner(&self) -> &SovTmConsensusState {
        match self {
            Self::Native { state } | Self::Wasm { state } => state,
        }
    }

    pub fn into_inner(self) -> SovTmConsensusState {
        match self {
            Self::Native { state } | Self::Wasm { state } => state,
        }
    }

    pub fn timestamp(&self) -> Time {
        self.inner().da_params.timestamp
    }

    pub fn next_validators_hash(&self) -> Hash {
        self.inner().da_params.next_validators_hash
    }

    pub fn native(state: SovTmConsensusState) -> Self {
        Self::Native { state }
    }

    pub fn wasm(state: SovTmConsensusState) -> Self {
        Self::Wasm { state }
    }
}

impl From<SovTmConsensusState> for ConsensusState {
    fn from(value: SovTmConsensusState) -> Self {
        Self::native(value)
    }
}

impl Protobuf<RawConsensusState> for ConsensusState {}

impl TryFrom<RawConsensusState> for ConsensusState {
    type Error = ClientError;

    fn try_from(raw: RawConsensusState) -> Result<Self, Self::Error> {
        Ok(Self::native(SovTmConsensusState::try_from(raw)?))
    }
}

impl From<ConsensusState> for RawConsensusState {
    fn from(client_state: ConsensusState) -> Self {
        client_state.into_inner().into()
    }
}

impl TryFrom<WasmConsensusState> for ConsensusState {
    type Error = ClientError;

    fn try_from(value: WasmConsensusState) -> Result<Self, Self::Error> {
        let any_data = Any::decode(value.data.as_slice()).map_err(|err| ClientError::Other {
            description: format!("Expected Any: {err}"),
        })?;
        Ok(Self::wasm(any_data.try_into()?))
    }
}

impl TryFrom<ConsensusState> for WasmConsensusState {
    type Error = ClientError;

    fn try_from(value: ConsensusState) -> Result<Self, Self::Error> {
        match value {
            ConsensusState::Wasm { state } => Ok(Self {
                data: Any::from(state).encode_to_vec(),
            }),
            _ => Err(ClientError::Other {
                description: "Wasm consensus state expected.".into(),
            }),
        }
    }
}

impl TryFrom<RawWasmConsensusState> for ConsensusState {
    type Error = ClientError;

    fn try_from(raw: RawWasmConsensusState) -> Result<Self, Self::Error> {
        let wasm_state = WasmConsensusState::try_from(raw)?;
        wasm_state.try_into()
    }
}

impl TryFrom<ConsensusState> for RawWasmConsensusState {
    type Error = ClientError;

    fn try_from(value: ConsensusState) -> Result<Self, Self::Error> {
        let wasm_state = WasmConsensusState::try_from(value)?;
        Ok(wasm_state.into())
    }
}

impl Protobuf<Any> for ConsensusState {}

impl TryFrom<Any> for ConsensusState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        WasmConsensusState::try_from(raw.clone())
            .and_then(|wasm_state| {
                let any_data =
                    Any::decode(wasm_state.data.as_slice()).map_err(|err| ClientError::Other {
                        description: format!("Expected Any: {err}"),
                    })?;
                Ok(Self::wasm(any_data.try_into()?))
            })
            .or_else(|_| SovTmConsensusState::try_from(raw).map(Self::native))
    }
}

impl From<ConsensusState> for Any {
    fn from(client_state: ConsensusState) -> Self {
        match client_state {
            ConsensusState::Native { state } => state.into(),
            ConsensusState::Wasm { state } => WasmConsensusState {
                data: Any::from(state).encode_to_vec(),
            }
            .into(),
        }
    }
}

impl ConsensusStateTrait for ConsensusState {
    fn root(&self) -> &CommitmentRoot {
        &self.inner().sovereign_params.root
    }

    fn timestamp(&self) -> Timestamp {
        let time = self.inner().da_params.timestamp.unix_timestamp_nanos();
        Timestamp::from_nanoseconds(time as u64).expect("invalid timestamp")
    }

    fn encode_vec(self) -> Vec<u8> {
        <Self as Protobuf<Any>>::encode_vec(self)
    }
}
