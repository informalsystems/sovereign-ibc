use ibc_core::client::context::{ClientExecutionContext, ClientValidationContext};
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::{
    iteration_key, ClientConsensusStatePath, ClientStatePath, ClientUpdateHeightPath,
    ClientUpdateTimePath,
};
use ibc_core::host::ValidationContext;
use ibc_core::primitives::Timestamp;
use sov_celestia_client::client_state::ClientState;

use super::Context;
use crate::types::AnyConsensusState;

impl ClientValidationContext for Context<'_> {
    fn update_meta(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<(Timestamp, Height), ContextError> {
        let client_update_time_path = ClientUpdateTimePath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        );

        let path_vec = client_update_time_path.leaf().into_bytes();

        let timestamp = self
            .retrieve(path_vec)
            .map(|timestamp| u64::from_be_bytes(timestamp.try_into().expect("invalid timestamp")));

        let timestamp = match timestamp {
            Some(time) => {
                Timestamp::from_nanoseconds(time).map_err(ClientError::InvalidPacketTimestamp)?
            }
            None => Err(ClientError::Other {
                description: "problem getting processed time".to_string(),
            })?,
        };

        let client_update_height_path = ClientUpdateHeightPath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        );

        let path_vec = client_update_height_path.leaf().into_bytes();

        let revision_height = self
            .retrieve(path_vec)
            .map(|height| u64::from_be_bytes(height.try_into().expect("invalid height")));

        let height = match revision_height {
            Some(h) => Height::new(0, h)?,
            None => Err(ClientError::Other {
                description: "problem getting processed time".to_string(),
            })?,
        };

        Ok((timestamp, height))
    }
}

impl ClientExecutionContext for Context<'_> {
    type V = <Self as ValidationContext>::V;
    type AnyClientState = <Self as ValidationContext>::AnyClientState;
    type AnyConsensusState = <Self as ValidationContext>::AnyConsensusState;

    fn store_client_state(
        &mut self,
        _client_state_path: ClientStatePath,
        client_state: ClientState,
    ) -> Result<(), ContextError> {
        let key = ClientStatePath::leaf().into_bytes();

        let encoded_client_state = self.encode_client_state(client_state)?;

        self.insert(key, encoded_client_state);

        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
        consensus_state: AnyConsensusState,
    ) -> Result<(), ContextError> {
        let key = consensus_state_path.leaf().into_bytes();

        let encoded_consensus_state = consensus_state.encode();

        self.insert(key, encoded_consensus_state);

        Ok(())
    }

    fn delete_consensus_state(
        &mut self,
        _consensus_state_path: ClientConsensusStatePath,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_update_meta(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_timestamp: Timestamp,
        host_height: Height,
    ) -> Result<(), ContextError> {
        let client_update_time_path = ClientUpdateTimePath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        );

        let path_vec = client_update_time_path.leaf().into_bytes();

        let time_vec: [u8; 8] = host_timestamp.nanoseconds().to_be_bytes();

        self.insert(path_vec, time_vec);

        let client_update_height_path = ClientUpdateHeightPath::new(
            client_id,
            height.revision_number(),
            height.revision_height(),
        );

        let path_vec = client_update_height_path.leaf().into_bytes();

        let revision_height_vec: [u8; 8] = host_height.revision_height().to_be_bytes();

        self.insert(path_vec, revision_height_vec);

        let iteration_key = iteration_key(height.revision_number(), height.revision_height());

        let height_vec = height.to_string().into_bytes();

        self.insert(iteration_key, height_vec);

        Ok(())
    }

    fn delete_update_meta(
        &mut self,
        _client_id: ClientId,
        _height: Height,
    ) -> Result<(), ContextError> {
        todo!()
    }
}
