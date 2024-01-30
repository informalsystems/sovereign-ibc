use ibc_client_tendermint::context::{
    CommonContext as TmCommonContext, ValidationContext as TmValidationContext,
};
use ibc_core::client::context::client_state::ClientStateCommon;
use ibc_core::client::context::{ClientExecutionContext, ClientValidationContext};
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::{ClientConsensusStatePath, ClientStatePath};
use ibc_core::host::ValidationContext;
use ibc_core::primitives::Timestamp;
use sov_celestia_client::context::{
    CommonContext as SovCommonContext, ValidationContext as SovValidationContext,
};
use sov_modules_api::{Context, DaSpec, StateMapAccessor, StateVecAccessor};

use super::AnyConsensusState;
use crate::context::IbcContext;

impl<'a, C: Context, Da: DaSpec> ClientValidationContext for IbcContext<'a, C, Da> {
    fn client_update_time(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Timestamp, ContextError> {
        self.ibc
            .client_update_host_times_map
            .get(
                &(client_id.clone(), *height),
                *self.working_set.borrow_mut(),
            )
            .ok_or(
                ClientError::Other {
                    description: "Client update time not found".to_string(),
                }
                .into(),
            )
    }

    fn client_update_height(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Height, ContextError> {
        self.ibc
            .client_update_host_heights_map
            .get(
                &(client_id.clone(), *height),
                *self.working_set.borrow_mut(),
            )
            .ok_or(
                ClientError::Other {
                    description: "Client update time not found".to_string(),
                }
                .into(),
            )
    }
}

impl<'a, C: Context, Da: DaSpec> ClientExecutionContext for IbcContext<'a, C, Da> {
    type V = <Self as ValidationContext>::V;
    type AnyClientState = <Self as ValidationContext>::AnyClientState;
    type AnyConsensusState = <Self as ValidationContext>::AnyConsensusState;

    fn store_client_state(
        &mut self,
        client_state_path: ClientStatePath,
        client_state: Self::AnyClientState,
    ) -> Result<(), ContextError> {
        self.ibc.client_state_map.set(
            &client_state_path.0,
            &client_state,
            &mut self.working_set.borrow_mut(),
        );

        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
        consensus_state: Self::AnyConsensusState,
    ) -> Result<(), ContextError> {
        self.ibc.consensus_state_map.set(
            &consensus_state_path,
            &consensus_state,
            &mut self.working_set.borrow_mut(),
        );

        Ok(())
    }

    fn delete_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
    ) -> Result<(), ContextError> {
        self.ibc
            .consensus_state_map
            .remove(&consensus_state_path, &mut self.working_set.borrow_mut());

        Ok(())
    }

    fn store_update_time(
        &mut self,
        client_id: ClientId,
        height: Height,
        timestamp: Timestamp,
    ) -> Result<(), ContextError> {
        self.ibc.client_update_host_times_map.set(
            &(client_id, height),
            &timestamp,
            *self.working_set.borrow_mut(),
        );
        Ok(())
    }

    fn store_update_height(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_height: Height,
    ) -> Result<(), ContextError> {
        self.ibc
            .client_update_heights_vec
            .push(&height, *self.working_set.borrow_mut());

        self.ibc.client_update_host_heights_map.set(
            &(client_id, height),
            &host_height,
            *self.working_set.borrow_mut(),
        );
        Ok(())
    }

    fn delete_update_time(
        &mut self,
        client_id: ClientId,
        height: Height,
    ) -> Result<(), ContextError> {
        self.ibc
            .client_update_host_times_map
            .remove(&(client_id, height), *self.working_set.borrow_mut());
        Ok(())
    }

    fn delete_update_height(
        &mut self,
        client_id: ClientId,
        height: Height,
    ) -> Result<(), ContextError> {
        self.ibc
            .client_update_host_heights_map
            .remove(&(client_id, height), *self.working_set.borrow_mut());
        Ok(())
    }
}

impl<'a, C: Context, Da: DaSpec> TmCommonContext for IbcContext<'a, C, Da> {
    type ConversionError = &'static str;
    type AnyConsensusState = AnyConsensusState;

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        <Self as ValidationContext>::host_timestamp(self)
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        <Self as ValidationContext>::host_height(self)
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        <Self as ValidationContext>::consensus_state(self, client_cons_state_path)
    }

    fn consensus_state_heights(&self, _client_id: &ClientId) -> Result<Vec<Height>, ContextError> {
        let heights = self
            .ibc
            .client_update_heights_vec
            .iter(*self.working_set.borrow_mut())
            .collect::<Vec<_>>();
        Ok(heights)
    }
}

impl<'a, C: Context, Da: DaSpec> TmValidationContext for IbcContext<'a, C, Da> {
    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        next_consensus_state(self, client_id, height)
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        prev_consensus_state(self, client_id, height)
    }
}

impl<'a, C: Context, Da: DaSpec> SovCommonContext for IbcContext<'a, C, Da> {
    type ConversionError = &'static str;
    type AnyConsensusState = AnyConsensusState;

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        <Self as ValidationContext>::host_timestamp(self)
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        <Self as ValidationContext>::host_height(self)
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        <Self as ValidationContext>::consensus_state(self, client_cons_state_path)
    }

    fn consensus_state_heights(&self, _client_id: &ClientId) -> Result<Vec<Height>, ContextError> {
        let heights = self
            .ibc
            .client_update_heights_vec
            .iter(*self.working_set.borrow_mut())
            .collect::<Vec<_>>();
        Ok(heights)
    }
}

impl<'a, C: Context, Da: DaSpec> SovValidationContext for IbcContext<'a, C, Da> {
    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        next_consensus_state(self, client_id, height)
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        prev_consensus_state(self, client_id, height)
    }
}

fn next_consensus_state<C: Context, D: DaSpec>(
    ctx: &IbcContext<'_, C, D>,
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

fn prev_consensus_state<C: Context, D: DaSpec>(
    ctx: &IbcContext<'_, C, D>,
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
