use ibc_client_tendermint::client_state::check_for_misbehaviour_on_misbehavior;
use ibc_client_tendermint::context::{DefaultVerifier, TmVerifier};
use ibc_core::client::context::client_state::ClientStateValidation;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Status;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::ClientConsensusStatePath;
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::proto::Any;
use sov_celestia_client_types::client_message::{
    SovTmHeader, SovTmMisbehaviour, SOV_TENDERMINT_HEADER_TYPE_URL,
    SOV_TENDERMINT_MISBEHAVIOUR_TYPE_URL,
};
use sov_celestia_client_types::client_state::SovTmClientState;

use super::ClientState;
use crate::client_state::{check_da_misbehaviour_on_update, verify_header, verify_misbehaviour};
use crate::context::{ConsensusStateConverter, ValidationContext as SovValidationContext};

impl<V> ClientStateValidation<V> for ClientState
where
    V: SovValidationContext,
    V::ConsensusStateRef: ConsensusStateConverter,
{
    fn verify_client_message(
        &self,
        ctx: &V,
        client_id: &ClientId,
        client_message: Any,
    ) -> Result<(), ClientError> {
        verify_client_message(
            self.inner(),
            ctx,
            client_id,
            client_message,
            &DefaultVerifier,
        )
    }

    fn check_for_misbehaviour(
        &self,
        ctx: &V,
        client_id: &ClientId,
        client_message: Any,
    ) -> Result<bool, ClientError> {
        check_for_misbehaviour(self.inner(), ctx, client_id, client_message)
    }

    fn status(&self, ctx: &V, client_id: &ClientId) -> Result<Status, ClientError> {
        status(self.inner(), ctx, client_id)
    }

    fn check_substitute(&self, ctx: &V, substitute_client_state: Any) -> Result<(), ClientError> {
        unimplemented!()
    }
}

/// Verify the client message as part of the validation process during the
/// update client flow.
pub fn verify_client_message<V>(
    client_state: &SovTmClientState,
    ctx: &V,
    client_id: &ClientId,
    client_message: Any,
    verifier: &impl TmVerifier,
) -> Result<(), ClientError>
where
    V: SovValidationContext,
    V::ConsensusStateRef: ConsensusStateConverter,
{
    match client_message.type_url.as_str() {
        SOV_TENDERMINT_HEADER_TYPE_URL => {
            let header = SovTmHeader::try_from(client_message)?;
            verify_header(
                ctx,
                &header,
                client_id,
                client_state.chain_id(),
                &client_state.as_light_client_options()?,
                verifier,
            )
        }
        SOV_TENDERMINT_MISBEHAVIOUR_TYPE_URL => {
            let misbehaviour = SovTmMisbehaviour::try_from(client_message)?;
            verify_misbehaviour(
                ctx,
                &misbehaviour,
                client_id,
                client_state.chain_id(),
                &client_state.as_light_client_options()?,
                verifier,
            )
        }
        _ => Err(ClientError::InvalidUpdateClientMessage),
    }
}

/// Check for misbehaviour on the client state as part of the client state
/// validation process.
pub fn check_for_misbehaviour<V>(
    client_state: &SovTmClientState,
    ctx: &V,
    client_id: &ClientId,
    client_message: Any,
) -> Result<bool, ClientError>
where
    V: SovValidationContext,
    V::ConsensusStateRef: ConsensusStateConverter,
{
    match client_message.type_url.as_str() {
        SOV_TENDERMINT_HEADER_TYPE_URL => {
            let header = SovTmHeader::try_from(client_message)?;
            check_da_misbehaviour_on_update(ctx, header, client_id, &client_state.latest_height)

            // TODO: Determine if we need any sort of misbehaviour check for the
            // rollup (aggregated proof) part.
        }
        SOV_TENDERMINT_MISBEHAVIOUR_TYPE_URL => {
            let misbehaviour = SovTmMisbehaviour::try_from(client_message)?;
            check_for_misbehaviour_on_misbehavior(
                &misbehaviour.header1().da_header,
                &misbehaviour.header2().da_header,
            )

            // TODO: Determine if we need any sort of misbehaviour check for the
            // rollup (aggregated proof) part.
        }
        _ => Err(ClientError::InvalidUpdateClientMessage),
    }
}

/// Checks the status (whether it is active, frozen, or expired) of the
/// Sovereign client state.
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
