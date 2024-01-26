use ibc_core::client::context::{ClientExecutionContext, ClientValidationContext};
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::{ClientConsensusStatePath, ClientStatePath};
use ibc_core::host::ValidationContext;
use ibc_core::primitives::proto::Any;
use ibc_core::primitives::Timestamp;

use super::definition::StorageMut;
use super::{Context, StorageRef};
use crate::types::{AnyClientState, AnyConsensusState, ProcessedStates, ReadonlyProcessedStates};

impl ClientValidationContext for Context<'_> {
    fn client_update_time(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Timestamp, ContextError> {
        client_update_time(self, client_id, height)
    }

    fn client_update_height(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Height, ContextError> {
        client_update_height(self, client_id, height)
    }
}

impl ClientExecutionContext for Context<'_> {
    type V = <Self as ValidationContext>::V;
    type AnyClientState = <Self as ValidationContext>::AnyClientState;
    type AnyConsensusState = <Self as ValidationContext>::AnyConsensusState;

    fn store_client_state(
        &mut self,
        client_state_path: ClientStatePath,
        client_state: AnyClientState,
    ) -> Result<(), ContextError> {
        store_client_state(self, client_state_path, client_state)
    }

    fn store_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
        consensus_state: AnyConsensusState,
    ) -> Result<(), ContextError> {
        store_consensus_state(self, consensus_state_path, consensus_state)
    }

    fn store_update_time(
        &mut self,
        client_id: ClientId,
        height: Height,
        timestamp: Timestamp,
    ) -> Result<(), ContextError> {
        store_update_time(self, client_id, height, timestamp)
    }

    fn store_update_height(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_height: Height,
    ) -> Result<(), ContextError> {
        store_update_height(self, client_id, height, host_height)
    }

    fn delete_consensus_state(
        &mut self,
        _consensus_state_path: ClientConsensusStatePath,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn delete_update_time(
        &mut self,
        _client_id: ClientId,
        _height: Height,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn delete_update_height(
        &mut self,
        _client_id: ClientId,
        _height: Height,
    ) -> Result<(), ContextError> {
        todo!()
    }
}

fn client_update_time<Ctx>(
    ctx: &Ctx,
    _client_id: &ClientId,
    height: &Height,
) -> Result<Timestamp, ContextError>
where
    Ctx: ClientValidationContext + StorageRef,
{
    let processed_state = ReadonlyProcessedStates::new(ctx.storage_ref());
    let timestamp = match processed_state.get_processed_time(*height, &mut Vec::new()) {
        Some(time) => {
            Timestamp::from_nanoseconds(time).map_err(ClientError::InvalidPacketTimestamp)?
        }
        None => Err(ClientError::Other {
            description: "problem getting processed time".to_string(),
        })?,
    };

    Ok(timestamp)
}

fn client_update_height<Ctx>(
    ctx: &Ctx,
    _client_id: &ClientId,
    height: &Height,
) -> Result<Height, ContextError>
where
    Ctx: ClientValidationContext + StorageRef,
{
    let processed_state = ReadonlyProcessedStates::new(ctx.storage_ref());

    let height = match processed_state.get_processed_height(*height, &mut Vec::new()) {
        Some(h) => Height::new(0, h)?,
        None => Err(ClientError::Other {
            description: "problem getting processed time".to_string(),
        })?,
    };

    Ok(height)
}

fn store_client_state<Ctx>(
    ctx: &mut Ctx,
    client_state_path: ClientStatePath,
    client_state: AnyClientState,
) -> Result<(), ContextError>
where
    Ctx: ClientExecutionContext + StorageMut,
{
    let client_state_value = Any::from(client_state).value;

    ctx.storage_mut().set(
        client_state_path.to_string().as_bytes(),
        client_state_value.as_slice(),
    );

    Ok(())
}

fn store_consensus_state<Ctx>(
    ctx: &mut Ctx,
    consensus_state_path: ClientConsensusStatePath,
    consensus_state: AnyConsensusState,
) -> Result<(), ContextError>
where
    Ctx: ClientExecutionContext + StorageMut,
{
    let consensus_state_value = Any::from(consensus_state).value;

    ctx.storage_mut().set(
        consensus_state_path.to_string().as_bytes(),
        consensus_state_value.as_slice(),
    );

    Ok(())
}

fn store_update_time<Ctx>(
    ctx: &mut Ctx,
    _client_id: ClientId,
    height: Height,
    timestamp: Timestamp,
) -> Result<(), ContextError>
where
    Ctx: ClientExecutionContext + StorageMut,
{
    let mut processed_state = ProcessedStates::new(ctx.storage_mut());
    processed_state.set_processed_time(height, timestamp.nanoseconds(), &mut Vec::new());

    Ok(())
}

fn store_update_height<Ctx>(
    ctx: &mut Ctx,
    _client_id: ClientId,
    height: Height,
    host_height: Height,
) -> Result<(), ContextError>
where
    Ctx: ClientExecutionContext + StorageMut,
{
    let mut processed_state = ProcessedStates::new(ctx.storage_mut());
    processed_state.set_processed_height(height, host_height.revision_height(), &mut Vec::new());
    processed_state.set_iteration_key(height, &mut Vec::new());
    Ok(())
}
