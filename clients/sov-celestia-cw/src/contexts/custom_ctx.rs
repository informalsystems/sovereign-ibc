use ibc_core::client::types::Height;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::ClientConsensusStatePath;
use ibc_core::host::ValidationContext as CoreValidationContext;
use ibc_core::primitives::Timestamp;
use sov_celestia_client::consensus_state::AnyConsensusState;
use sov_celestia_client::context::{CommonContext, ValidationContext};
use sov_celestia_client::error::Error;

use super::definition::ContextMut;
use super::{ContextRef, StorageRef};
use crate::types::processed_states::ReadonlyProcessedStates;

impl CommonContext for ContextMut<'_> {
    type ConversionError = Error;
    type AnyConsensusState = AnyConsensusState;

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        CoreValidationContext::host_timestamp(self)
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        CoreValidationContext::host_height(self)
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        CoreValidationContext::consensus_state(self, client_cons_state_path)
    }
}

impl ValidationContext for ContextMut<'_> {
    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        let processed_state = ReadonlyProcessedStates::new(self.storage());
        match processed_state.get_next_height(*height) {
            Some(next_height) => {
                let cons_state_path = ClientConsensusStatePath::new(
                    client_id.clone(),
                    next_height.revision_number(),
                    next_height.revision_height(),
                );
                CoreValidationContext::consensus_state(self, &cons_state_path).map(Some)
            }
            None => Ok(None),
        }
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        let processed_state = ReadonlyProcessedStates::new(self.storage());
        match processed_state.get_prev_height(*height) {
            Some(prev_height) => {
                let cons_state_path = ClientConsensusStatePath::new(
                    client_id.clone(),
                    prev_height.revision_number(),
                    prev_height.revision_height(),
                );
                CoreValidationContext::consensus_state(self, &cons_state_path).map(Some)
            }
            None => Ok(None),
        }
    }
}

impl CommonContext for ContextRef<'_> {
    type ConversionError = Error;
    type AnyConsensusState = AnyConsensusState;

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        CoreValidationContext::host_timestamp(self)
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        CoreValidationContext::host_height(self)
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        CoreValidationContext::consensus_state(self, client_cons_state_path)
    }
}

impl ValidationContext for ContextRef<'_> {
    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        let processed_state = ReadonlyProcessedStates::new(self.storage());
        match processed_state.get_next_height(*height) {
            Some(next_height) => {
                let cons_state_path = ClientConsensusStatePath::new(
                    client_id.clone(),
                    next_height.revision_number(),
                    next_height.revision_height(),
                );
                CoreValidationContext::consensus_state(self, &cons_state_path).map(Some)
            }
            None => Ok(None),
        }
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        let processed_state = ReadonlyProcessedStates::new(self.storage());
        match processed_state.get_prev_height(*height) {
            Some(prev_height) => {
                let cons_state_path = ClientConsensusStatePath::new(
                    client_id.clone(),
                    prev_height.revision_number(),
                    prev_height.revision_height(),
                );
                CoreValidationContext::consensus_state(self, &cons_state_path).map(Some)
            }
            None => Ok(None),
        }
    }
}
