use ibc::core::ics02_client::error::ClientError;
use ibc::core::ics02_client::{ClientExecutionContext, ClientValidationContext};
use ibc::core::ics24_host::identifier::ClientId;
use ibc::core::ics24_host::path::{ClientConsensusStatePath, ClientStatePath};
use ibc::core::timestamp::Timestamp;
use ibc::core::{ContextError, ValidationContext};
use ibc::proto::Any;
use ibc::Height;
use ics11_sov_celestia::client_state::AnyClientState;
use ics11_sov_celestia::consensus_state::AnyConsensusState;

use super::definition::{ContextMut, StorageMut};
use super::{ContextRef, StorageRef};
use crate::contract::processed_states::{ProcessedStates, ReadonlyProcessedStates};

impl ClientValidationContext for ContextMut<'_> {
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

impl ClientExecutionContext for ContextMut<'_> {
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
        _client_id: ClientId,
        height: Height,
        timestamp: Timestamp,
    ) -> Result<(), ContextError> {
        store_update_time(self, _client_id, height, timestamp)
    }

    fn store_update_height(
        &mut self,
        _client_id: ClientId,
        height: Height,
        host_height: Height,
    ) -> Result<(), ContextError> {
        store_update_height(self, _client_id, height, host_height)
    }
}

impl ClientValidationContext for ContextRef<'_> {
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

impl ClientExecutionContext for ContextRef<'_> {
    type V = <Self as ValidationContext>::V;
    type AnyClientState = <Self as ValidationContext>::AnyClientState;
    type AnyConsensusState = <Self as ValidationContext>::AnyConsensusState;

    fn store_client_state(
        &mut self,
        _client_state_path: ClientStatePath,
        _client_state: AnyClientState,
    ) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn store_consensus_state(
        &mut self,
        _consensus_state_path: ClientConsensusStatePath,
        _consensus_state: AnyConsensusState,
    ) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn store_update_time(
        &mut self,
        _client_id: ClientId,
        _height: Height,
        _timestamp: Timestamp,
    ) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn store_update_height(
        &mut self,
        _client_id: ClientId,
        _height: Height,
        _host_height: Height,
    ) -> Result<(), ContextError> {
        unimplemented!()
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
    let processed_state = ReadonlyProcessedStates::new(ctx.storage());
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
    let processed_state = ReadonlyProcessedStates::new(ctx.storage());

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
