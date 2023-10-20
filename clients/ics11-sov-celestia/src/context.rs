use alloc::string::ToString;

use ibc::core::ics02_client::ClientExecutionContext;
use ibc::core::ics24_host::identifier::ClientId;
use ibc::core::ics24_host::path::ClientConsensusStatePath;
use ibc::core::timestamp::Timestamp;
use ibc::core::ContextError;
use ibc::Height;

use crate::consensus_state::SovConsensusState;

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
