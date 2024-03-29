use ibc_client_wasm_types::client_state::ClientState as WasmClientState;
use ibc_client_wasm_types::consensus_state::ConsensusState as WasmConsensusState;
use ibc_core::client::context::{ClientExecutionContext, ClientValidationContext};
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::{iteration_key, ClientConsensusStatePath, ClientStatePath};
use ibc_core::primitives::proto::{Any, Protobuf};
use ibc_core::primitives::Timestamp;
use sov_celestia_client::types::codec::AnyCodec;

use super::Context;
use crate::types::ClientType;

impl<'a, C: ClientType<'a>> ClientValidationContext for Context<'a, C> {
    type ClientStateRef = C::ClientState;
    type ConsensusStateRef = C::ConsensusState;

    fn client_state(&self, _client_id: &ClientId) -> Result<Self::ClientStateRef, ContextError> {
        let client_state_value = self.retrieve(ClientStatePath::leaf())?;

        let any_wasm: WasmClientState = Protobuf::<Any>::decode(client_state_value.as_slice())
            .map_err(|e| ClientError::Other {
                description: e.to_string(),
            })?;

        let sov_client_state = C::ClientState::decode_thru_any(any_wasm.data)?;

        Ok(sov_client_state)
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::ConsensusStateRef, ContextError> {
        let consensus_state_value = self.retrieve(client_cons_state_path.leaf())?;

        let any_wasm: WasmConsensusState =
            C::ConsensusState::decode_thru_any(consensus_state_value)?;

        let consensus_state = C::ConsensusState::decode_thru_any(any_wasm.data)?;

        Ok(consensus_state)
    }

    fn client_update_meta(
        &self,
        _client_id: &ClientId,
        height: &Height,
    ) -> Result<(Timestamp, Height), ContextError> {
        let time_key = self.client_update_time_key(height);

        let time_vec = self.retrieve(time_key)?;

        let time = u64::from_be_bytes(time_vec.try_into().expect("invalid timestamp"));

        let timestamp =
            Timestamp::from_nanoseconds(time).map_err(ClientError::InvalidPacketTimestamp)?;

        let height_key = self.client_update_height_key(height);

        let revision_height_vec = self.retrieve(height_key)?;

        let revision_height =
            u64::from_be_bytes(revision_height_vec.try_into().expect("invalid height"));

        let height = Height::new(0, revision_height)?;

        Ok((timestamp, height))
    }
}

impl<'a, C: ClientType<'a>> ClientExecutionContext for Context<'a, C> {
    type ClientStateMut = C::ClientState;

    fn store_client_state(
        &mut self,
        _client_state_path: ClientStatePath,
        client_state: Self::ClientStateMut,
    ) -> Result<(), ContextError> {
        let key = ClientStatePath::leaf().into_bytes();

        let encoded_client_state = self.encode_client_state(client_state)?;

        self.insert(key, encoded_client_state);

        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
        consensus_state: Self::ConsensusStateRef,
    ) -> Result<(), ContextError> {
        let key = consensus_state_path.leaf().into_bytes();

        let encoded_consensus_state = C::ConsensusState::encode_thru_any(consensus_state);

        let wasm_consensus_state = WasmConsensusState {
            data: encoded_consensus_state,
        };

        let encoded_wasm_consensus_state = C::ConsensusState::encode_thru_any(wasm_consensus_state);

        self.insert(key, encoded_wasm_consensus_state);

        Ok(())
    }

    fn delete_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
    ) -> Result<(), ContextError> {
        self.remove(consensus_state_path.leaf().into_bytes());

        Ok(())
    }

    fn store_update_meta(
        &mut self,
        _client_id: ClientId,
        height: Height,
        host_timestamp: Timestamp,
        host_height: Height,
    ) -> Result<(), ContextError> {
        let time_key = self.client_update_time_key(&height);

        let time_vec: [u8; 8] = host_timestamp.nanoseconds().to_be_bytes();

        self.insert(time_key, time_vec);

        let height_key = self.client_update_height_key(&height);

        let revision_height_vec: [u8; 8] = host_height.revision_height().to_be_bytes();

        self.insert(height_key, revision_height_vec);

        let iteration_key = iteration_key(height.revision_number(), height.revision_height());

        let height_vec = height.to_string().into_bytes();

        self.insert(iteration_key, height_vec);

        Ok(())
    }

    fn delete_update_meta(
        &mut self,
        _client_id: ClientId,
        height: Height,
    ) -> Result<(), ContextError> {
        let time_key = self.client_update_time_key(&height);

        self.remove(time_key);

        let height_key = self.client_update_height_key(&height);

        self.remove(height_key);

        let iteration_key = iteration_key(height.revision_number(), height.revision_height());

        self.remove(iteration_key);

        Ok(())
    }
}
