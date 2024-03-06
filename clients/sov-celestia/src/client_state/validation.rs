use ibc_core::client::context::client_state::ClientStateValidation;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Status;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::ClientConsensusStatePath;
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::proto::Any;
use sov_celestia_client_types::client_state::SovTmClientState;

use super::ClientState;
use crate::context::{ConsensusStateConverter, ValidationContext as SovValidationContext};

impl<V> ClientStateValidation<V> for ClientState
where
    V: SovValidationContext,
    V::ConsensusStateRef: ConsensusStateConverter,
{
    fn verify_client_message(
        &self,
        _ctx: &V,
        _client_id: &ClientId,
        _client_message: Any,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn check_for_misbehaviour(
        &self,
        _ctx: &V,
        _client_id: &ClientId,
        _client_message: Any,
    ) -> Result<bool, ClientError> {
        Ok(false)
    }

    fn status(&self, ctx: &V, client_id: &ClientId) -> Result<Status, ClientError> {
        status(self.inner(), ctx, client_id)
    }
}

pub fn status<V>(
    client_state: &SovTmClientState,
    ctx: &V,
    client_id: &ClientId,
) -> Result<Status, ClientError>
where
    V: SovValidationContext,
    V::ConsensusStateRef: ConsensusStateConverter,
{
    if client_state.is_frozen() {
        return Ok(Status::Frozen);
    }

    let latest_consensus_state = {
        match ctx.consensus_state(&ClientConsensusStatePath::new(
            client_id.clone(),
            client_state.latest_height.revision_number(),
            client_state.latest_height.revision_height(),
        )) {
            Ok(cs) => cs.try_into()?,
            // if the client state does not have an associated consensus state for its latest height
            // then it must be expired
            Err(_) => return Ok(Status::Expired),
        }
    };

    // Note: if the `duration_since()` is `None`, indicating that the latest
    // consensus state is in the future, then we don't consider the client
    // to be expired.
    let now = ctx.host_timestamp()?;
    if let Some(elapsed_since_latest_consensus_state) =
        now.duration_since(&latest_consensus_state.timestamp().into())
    {
        if elapsed_since_latest_consensus_state > client_state.da_params.trusting_period {
            return Ok(Status::Expired);
        }
    }

    Ok(Status::Active)
}
