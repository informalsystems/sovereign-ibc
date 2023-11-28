use ibc_client_tendermint::context::{
    CommonContext as TmCommonContext, ValidationContext as TmValidationContext,
};
use ibc_core::client::context::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation,
};
use ibc_core::client::context::{ClientExecutionContext, ClientValidationContext};
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::{Height, Status, UpdateKind};
use ibc_core::commitment_types::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::{ClientId, ClientType};
use ibc_core::host::types::path::{ClientConsensusStatePath, ClientStatePath, Path};
use ibc_core::host::ValidationContext;
use ibc_core::primitives::proto::Any;
use ibc_core::primitives::Timestamp;
use sov_modules_api::{Context, DaSpec};

use super::{AnyClientState, AnyConsensusState};
use crate::context::IbcContext;

// Next 3 trait impls are boilerplate
// We have a `ClientState` macro, but unfortunately it doesn't currently support
// the context (`IbcExecutionContext` in this case) to be generic
impl ClientStateCommon for AnyClientState {
    fn verify_consensus_state(&self, consensus_state: Any) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.verify_consensus_state(consensus_state),
        }
    }

    fn client_type(&self) -> ClientType {
        match self {
            AnyClientState::Tendermint(cs) => cs.client_type(),
        }
    }

    fn latest_height(&self) -> Height {
        match self {
            AnyClientState::Tendermint(cs) => cs.latest_height(),
        }
    }

    fn validate_proof_height(&self, proof_height: Height) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.validate_proof_height(proof_height),
        }
    }

    fn verify_upgrade_client(
        &self,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
        proof_upgrade_client: CommitmentProofBytes,
        proof_upgrade_consensus_state: CommitmentProofBytes,
        root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.verify_upgrade_client(
                upgraded_client_state,
                upgraded_consensus_state,
                proof_upgrade_client,
                proof_upgrade_consensus_state,
                root,
            ),
        }
    }

    fn verify_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
        value: Vec<u8>,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => {
                cs.verify_membership(prefix, proof, root, path, value)
            }
        }
    }

    fn verify_non_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.verify_non_membership(prefix, proof, root, path),
        }
    }
}

impl<'a, C, Da> ClientStateExecution<IbcContext<'a, C, Da>> for AnyClientState
where
    C: Context,
    Da: DaSpec,
{
    fn initialise(
        &self,
        ctx: &mut IbcContext<'a, C, Da>,
        client_id: &ClientId,
        consensus_state: Any,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.initialise(ctx, client_id, consensus_state),
        }
    }

    fn update_state(
        &self,
        ctx: &mut IbcContext<'a, C, Da>,
        client_id: &ClientId,
        header: Any,
    ) -> Result<Vec<Height>, ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.update_state(ctx, client_id, header),
        }
    }

    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut IbcContext<'a, C, Da>,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => {
                cs.update_state_on_misbehaviour(ctx, client_id, client_message, update_kind)
            }
        }
    }

    fn update_state_on_upgrade(
        &self,
        ctx: &mut IbcContext<'a, C, Da>,
        client_id: &ClientId,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
    ) -> Result<Height, ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.update_state_on_upgrade(
                ctx,
                client_id,
                upgraded_client_state,
                upgraded_consensus_state,
            ),
        }
    }
}

impl<'a, C, Da> ClientStateValidation<IbcContext<'a, C, Da>> for AnyClientState
where
    C: Context,
    Da: DaSpec,
{
    fn verify_client_message(
        &self,
        ctx: &IbcContext<'a, C, Da>,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => {
                cs.verify_client_message(ctx, client_id, client_message, update_kind)
            }
        }
    }

    fn check_for_misbehaviour(
        &self,
        ctx: &IbcContext<'a, C, Da>,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<bool, ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => {
                cs.check_for_misbehaviour(ctx, client_id, client_message, update_kind)
            }
        }
    }

    fn status(
        &self,
        ctx: &IbcContext<'a, C, Da>,
        client_id: &ClientId,
    ) -> Result<Status, ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.status(ctx, client_id),
        }
    }
}

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
        // Searches for the most recent height at which a client has been
        // updated and a consensus state has been stored.
        let latest_height = self
            .ibc
            .client_state_map
            .get(client_id, *self.working_set.borrow_mut())
            .map(|cs| cs.latest_height())
            .ok_or(ClientError::ClientStateNotFound {
                client_id: client_id.clone(),
            })?;

        if height.revision_number() != latest_height.revision_number() {
            return Err(ClientError::Other {
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

            let next_cons_state = self
                .ibc
                .consensus_state_map
                .get(&cons_state_path, *self.working_set.borrow_mut());

            if next_cons_state.is_some() {
                return Ok(next_cons_state);
            }
        }

        Ok(None)
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        // Searches for the most recent height at which a client has been
        // updated and a consensus state has been stored.
        let latest_height = self
            .ibc
            .client_state_map
            .get(client_id, *self.working_set.borrow_mut())
            .map(|cs| cs.latest_height())
            .ok_or(ClientError::ClientStateNotFound {
                client_id: client_id.clone(),
            })?;

        if height.revision_number() != latest_height.revision_number() {
            return Err(ClientError::Other {
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

            let prev_cons_state = self
                .ibc
                .consensus_state_map
                .get(&cons_state_path, *self.working_set.borrow_mut());

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

            let prev_cons_state = self
                .ibc
                .consensus_state_map
                .get(&cons_state_path, *self.working_set.borrow_mut());

            if prev_cons_state.is_some() {
                return Ok(prev_cons_state);
            }

            target_height = target_height.decrement()?;
        }

        Ok(None)
    }
}
