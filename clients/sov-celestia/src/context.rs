use ibc_core::client::context::{ClientExecutionContext, ClientValidationContext};
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::Timestamp;
use sov_celestia_client_types::consensus_state::SovTmConsensusState;

/// Enables conversion (`TryInto` and `From`) between the consensus state type
/// used by the host and the one specific to the Sovereign light client, which
/// is `SovTmConsensusState`.
pub trait ConsensusStateConverter:
    TryInto<SovTmConsensusState, Error = ClientError> + From<SovTmConsensusState>
{
}

impl<C> ConsensusStateConverter for C where
    C: TryInto<SovTmConsensusState, Error = ClientError> + From<SovTmConsensusState>
{
}

/// Client's context required during validation
pub trait ValidationContext: ClientValidationContext
where
    Self::ConsensusStateRef: ConsensusStateConverter,
{
    /// Returns the current timestamp of the local chain.
    fn host_timestamp(&self) -> Result<Timestamp, ContextError>;

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Result<Height, ContextError>;

    /// Returns all the heights at which a consensus state is stored
    fn consensus_state_heights(&self, client_id: &ClientId) -> Result<Vec<Height>, ContextError>;

    /// Search for the lowest consensus state higher than `height`.
    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ContextError>;

    /// Search for the highest consensus state lower than `height`.
    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ContextError>;
}

/// Client's context required during execution.
///
/// This trait is automatically implemented for all types that implement
/// [`ValidationContext`] and [`ClientExecutionContext`]
pub trait ExecutionContext: ValidationContext + ClientExecutionContext
where
    Self::ConsensusStateRef: ConsensusStateConverter,
{
}

impl<T> ExecutionContext for T
where
    T: ValidationContext + ClientExecutionContext,
    T::ConsensusStateRef: ConsensusStateConverter,
{
}
