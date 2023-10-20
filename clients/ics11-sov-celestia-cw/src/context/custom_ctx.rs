use ibc::core::ics24_host::identifier::ClientId;
use ibc::core::ics24_host::path::ClientConsensusStatePath;
use ibc::core::timestamp::Timestamp;
use ibc::core::{ContextError, ValidationContext as CoreValidationContext};
use ibc::Height;
use ics11_sov_celestia::consensus_state::AnyConsensusState;
use ics11_sov_celestia::context::{CommonContext, ValidationContext};
use ics11_sov_celestia::error::Error;

use super::definition::ContextMut;
use super::{ContextRef, StorageRef};
use crate::contract::processed_states::ReadonlyProcessedStates;

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
                let cons_state_path = ClientConsensusStatePath::new(client_id, &next_height);
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
                let cons_state_path = ClientConsensusStatePath::new(client_id, &prev_height);
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
                let cons_state_path = ClientConsensusStatePath::new(client_id, &next_height);
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
                let cons_state_path = ClientConsensusStatePath::new(client_id, &prev_height);
                CoreValidationContext::consensus_state(self, &cons_state_path).map(Some)
            }
            None => Ok(None),
        }
    }
}
