use ibc_client_tendermint::context::ValidationContext as TmValidationContext;
use ibc_core::client::context::prelude::*;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::{ClientConsensusStatePath, ClientStatePath};
use ibc_core::host::ValidationContext;
use ibc_core::primitives::Timestamp;
use sov_celestia_client::context::ValidationContext as SovValidationContext;
use sov_modules_api::{DaSpec, Spec, StateMapAccessor, StateVecAccessor};

use super::{AnyClientState, AnyConsensusState};
use crate::context::IbcContext;

impl<'a, S: Spec, Da: DaSpec> ClientValidationContext for IbcContext<'a, S, Da> {
    type ClientStateRef = AnyClientState;
    type ConsensusStateRef = AnyConsensusState;

    fn client_state(&self, client_id: &ClientId) -> Result<Self::ClientStateRef, ContextError> {
        self.ibc
            .client_state_map
            .get(client_id, *self.working_set.borrow_mut())
            .ok_or(
                ClientError::ClientStateNotFound {
                    client_id: client_id.clone(),
                }
                .into(),
            )
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::ConsensusStateRef, ContextError> {
        self.ibc
            .consensus_state_map
            .get(client_cons_state_path, *self.working_set.borrow_mut())
            .ok_or(
                ClientError::ConsensusStateNotFound {
                    client_id: client_cons_state_path.client_id.clone(),
                    height: Height::new(
                        client_cons_state_path.revision_number,
                        client_cons_state_path.revision_height,
                    )
                    .map_err(|_| ClientError::Other {
                        description: "Height cannot be zero".to_string(),
                    })?,
                }
                .into(),
            )
    }

    fn client_update_meta(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<(Timestamp, Height), ContextError> {
        let update_meta = self
            .ibc
            .client_update_meta_map
            .get(
                &(client_id.clone(), *height),
                *self.working_set.borrow_mut(),
            )
            .ok_or(ClientError::UpdateMetaDataNotFound {
                client_id: client_id.clone(),
                height: *height,
            })?;

        Ok(update_meta)
    }
}

impl<'a, S: Spec, Da: DaSpec> ClientExecutionContext for IbcContext<'a, S, Da> {
    type ClientStateMut = AnyClientState;

    fn store_client_state(
        &mut self,
        client_state_path: ClientStatePath,
        client_state: Self::ClientStateMut,
    ) -> Result<(), ContextError> {
        self.ibc.client_state_map.set(
            &client_state_path.0,
            &client_state,
            *self.working_set.borrow_mut(),
        );

        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
        consensus_state: Self::ConsensusStateRef,
    ) -> Result<(), ContextError> {
        self.ibc.consensus_state_map.set(
            &consensus_state_path,
            &consensus_state,
            *self.working_set.borrow_mut(),
        );

        Ok(())
    }

    fn delete_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
    ) -> Result<(), ContextError> {
        self.ibc
            .consensus_state_map
            .remove(&consensus_state_path, *self.working_set.borrow_mut());

        Ok(())
    }

    fn store_update_meta(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_timestamp: Timestamp,
        host_height: Height,
    ) -> Result<(), ContextError> {
        self.ibc
            .client_update_heights_vec
            .push(&height, *self.working_set.borrow_mut());

        self.ibc.client_update_meta_map.set(
            &(client_id.clone(), height),
            &(host_timestamp, host_height),
            *self.working_set.borrow_mut(),
        );

        Ok(())
    }

    fn delete_update_meta(
        &mut self,
        client_id: ClientId,
        height: Height,
    ) -> Result<(), ContextError> {
        self.ibc
            .client_update_meta_map
            .remove(&(client_id.clone(), height), *self.working_set.borrow_mut());
        Ok(())
    }
}

impl<'a, S: Spec, Da: DaSpec> TmValidationContext for IbcContext<'a, S, Da> {
    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        <Self as ValidationContext>::host_timestamp(self)
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        <Self as ValidationContext>::host_height(self)
    }

    fn consensus_state_heights(&self, _client_id: &ClientId) -> Result<Vec<Height>, ContextError> {
        let heights = self
            .ibc
            .client_update_heights_vec
            .iter(*self.working_set.borrow_mut())
            .collect::<Vec<_>>();
        Ok(heights)
    }

    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ContextError> {
        next_consensus_state(self, client_id, height)
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ContextError> {
        prev_consensus_state(self, client_id, height)
    }
}

impl<'a, S: Spec, Da: DaSpec> SovValidationContext for IbcContext<'a, S, Da> {
    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        <Self as ValidationContext>::host_timestamp(self)
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        <Self as ValidationContext>::host_height(self)
    }

    fn consensus_state_heights(&self, _client_id: &ClientId) -> Result<Vec<Height>, ContextError> {
        let heights = self
            .ibc
            .client_update_heights_vec
            .iter(*self.working_set.borrow_mut())
            .collect::<Vec<_>>();
        Ok(heights)
    }

    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ContextError> {
        next_consensus_state(self, client_id, height)
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ContextError> {
        prev_consensus_state(self, client_id, height)
    }
}

fn next_consensus_state<S: Spec, D: DaSpec>(
    ctx: &IbcContext<'_, S, D>,
    client_id: &ClientId,
    height: &Height,
) -> Result<Option<AnyConsensusState>, ContextError> {
    // Searches for the most recent height at which a client has been
    // updated and a consensus state has been stored.
    let latest_height = ctx
        .ibc
        .client_state_map
        .get(client_id, *ctx.working_set.borrow_mut())
        .map(|cs| cs.latest_height())
        .ok_or(ClientError::ClientStateNotFound {
            client_id: client_id.clone(),
        })?;

    if height.revision_number() != latest_height.revision_number() {
        Err(ClientError::Other {
            description: "height revision number must match the chain's revision number"
                .to_string(),
        })?;
    }

    // If the height is greater equal than the latest height, there is no
    // next consensus state
    if height >= &latest_height {
        return Ok(None);
    }

    // Otherwise, we iterate over the heights between the given height and
    // the latest height, and return the first consensus state we find
    //
    // NOTE: this is not the efficient way to do this, but no other way at
    // the moment as we don't have access to an iterator over the map keys

    let mut target_height = *height;

    while height.revision_height() < latest_height.revision_height() {
        target_height = target_height.increment();

        let cons_state_path = ClientConsensusStatePath::new(
            client_id.clone(),
            target_height.revision_number(),
            target_height.revision_height(),
        );

        let next_cons_state = ctx
            .ibc
            .consensus_state_map
            .get(&cons_state_path, *ctx.working_set.borrow_mut());

        if next_cons_state.is_some() {
            return Ok(next_cons_state);
        }
    }

    Ok(None)
}

fn prev_consensus_state<S: Spec, D: DaSpec>(
    ctx: &IbcContext<'_, S, D>,
    client_id: &ClientId,
    height: &Height,
) -> Result<Option<AnyConsensusState>, ContextError> {
    // Searches for the most recent height at which a client has been
    // updated and a consensus state has been stored.
    let latest_height = ctx
        .ibc
        .client_state_map
        .get(client_id, *ctx.working_set.borrow_mut())
        .map(|cs| cs.latest_height())
        .ok_or(ClientError::ClientStateNotFound {
            client_id: client_id.clone(),
        })?;

    if height.revision_number() != latest_height.revision_number() {
        Err(ClientError::Other {
            description: "height revision number must match the chain's revision number"
                .to_string(),
        })?;
    }

    // If the height is greater equal than the latest height, the previous
    // consensus state is the latest consensus state
    if height >= &latest_height {
        let cons_state_path = ClientConsensusStatePath::new(
            client_id.clone(),
            latest_height.revision_number(),
            latest_height.revision_height(),
        );

        let prev_cons_state = ctx
            .ibc
            .consensus_state_map
            .get(&cons_state_path, *ctx.working_set.borrow_mut());

        return Ok(prev_cons_state);
    }

    // Otherwise, we decrement the height until we reach the first consensus
    // state we find
    //
    // NOTE: this is not the efficient way to do this, but no other way at
    // the moment as we don't have access to an iterator over the map keys
    let mut target_height = *height;

    while target_height.revision_height() > 0 {
        let cons_state_path = ClientConsensusStatePath::new(
            client_id.clone(),
            target_height.revision_number(),
            target_height.revision_height(),
        );

        let prev_cons_state = ctx
            .ibc
            .consensus_state_map
            .get(&cons_state_path, *ctx.working_set.borrow_mut());

        if prev_cons_state.is_some() {
            return Ok(prev_cons_state);
        }

        target_height = target_height.decrement()?;
    }

    Ok(None)
}
