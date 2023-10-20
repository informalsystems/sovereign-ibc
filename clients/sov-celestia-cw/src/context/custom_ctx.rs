use alloc::string::ToString;

use ibc::core::ics02_client::ClientExecutionContext;
use ibc::core::ics24_host::identifier::ClientId;
use ibc::core::ics24_host::path::ClientConsensusStatePath;
use ibc::core::timestamp::Timestamp;
use ibc::core::{ContextError, ValidationContext as CoreValidationContext};
use ibc::Height;

use super::definition::ContextMut;
use super::{ContextRef, StorageRef};
use crate::contract::processed_states::ReadonlyProcessedStates;
use crate::types::consensus_state::{AnyConsensusState, SovConsensusState};
use crate::types::error::Error;

/// Client's context required during both validation and execution
pub trait CommonContext {
    type ConversionError: ToString;
    type AnyConsensusState: TryInto<SovConsensusState, Error = Self::ConversionError>;

    /// Returns the current timestamp of the local chain.
    fn host_timestamp(&self) -> Result<Timestamp, ContextError>;

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Result<Height, ContextError>;

    /// Retrieve the consensus state for the given client ID at the specified
    /// height.
    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError>;
}

/// Client's context required during validation
pub trait ValidationContext: CommonContext {
    /// Search for the lowest consensus state higher than `height`.
    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError>;

    /// Search for the highest consensus state lower than `height`.
    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError>;
}

/// Client's context required during execution.
///
/// This trait is automatically implemented for all types that implement
/// [`CommonContext`] and [`ClientExecutionContext`]
pub trait ExecutionContext: CommonContext + ClientExecutionContext {}

impl<T> ExecutionContext for T where T: CommonContext + ClientExecutionContext {}

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
