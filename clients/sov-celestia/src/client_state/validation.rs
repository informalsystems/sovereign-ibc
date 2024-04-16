use ibc_client_tendermint::client_state::check_for_misbehaviour_on_misbehavior;
use ibc_client_tendermint::verifier::{DefaultVerifier, TmVerifier};
use ibc_core::client::context::client_state::ClientStateValidation;
use ibc_core::client::context::{Convertible, ExtClientValidationContext};
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
use sov_celestia_client_types::consensus_state::SovTmConsensusState;

use super::ClientState;
use crate::client_state::{check_da_misbehaviour_on_update, verify_header, verify_misbehaviour};

impl<V> ClientStateValidation<V> for ClientState
where
    V: ExtClientValidationContext,
    V::ConsensusStateRef: Convertible<SovTmConsensusState, ClientError>,
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
        check_substitute::<V>(self.inner(), substitute_client_state)
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
    V: ExtClientValidationContext,
    V::ConsensusStateRef: Convertible<SovTmConsensusState, ClientError>,
{
    match client_message.type_url.as_str() {
        SOV_TENDERMINT_HEADER_TYPE_URL => {
            let header = SovTmHeader::try_from(client_message)?;
            verify_header(ctx, client_state, &header, client_id, verifier)
        }
        SOV_TENDERMINT_MISBEHAVIOUR_TYPE_URL => {
            let misbehaviour = SovTmMisbehaviour::try_from(client_message)?;
            verify_misbehaviour(ctx, client_state, &misbehaviour, client_id, verifier)
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
    V: ExtClientValidationContext,
    V::ConsensusStateRef: Convertible<SovTmConsensusState, ClientError>,
{
    match client_message.type_url.as_str() {
        SOV_TENDERMINT_HEADER_TYPE_URL => {
            let header = SovTmHeader::try_from(client_message)?;
            check_da_misbehaviour_on_update(
                ctx,
                header,
                client_id,
                &client_state.latest_height_in_sov(),
            )
        }
        SOV_TENDERMINT_MISBEHAVIOUR_TYPE_URL => {
            let misbehaviour = SovTmMisbehaviour::try_from(client_message)?;
            check_for_misbehaviour_on_misbehavior(
                &misbehaviour.header_1().da_header,
                &misbehaviour.header_2().da_header,
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
    V: ExtClientValidationContext,
    V::ConsensusStateRef: Convertible<SovTmConsensusState, ClientError>,
{
    if client_state.is_frozen() {
        return Ok(Status::Frozen);
    }

    let latest_consensus_state = {
        match ctx.consensus_state(&ClientConsensusStatePath::new(
            client_id.clone(),
            client_state.latest_height_in_sov().revision_number(),
            client_state.latest_height_in_sov().revision_height(),
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
        if elapsed_since_latest_consensus_state > client_state.trusting_period() {
            return Ok(Status::Expired);
        }
    }

    Ok(Status::Active)
}

/// the client recovery validation step.
///
/// The subject and substitute client states match if all their respective
/// client state parameters match except for frozen height, latest height,
/// trusting periods, and chain ID.
pub fn check_substitute<V>(
    subject_client_state: &SovTmClientState,
    substitute_client_state: Any,
) -> Result<(), ClientError>
where
    V: ExtClientValidationContext,
    V::ConsensusStateRef: Convertible<SovTmConsensusState, ClientError>,
{
    let substitute_client_state = SovTmClientState::try_from(substitute_client_state)?;

    let sov_params_matches = subject_client_state
        .sovereign_params
        .check_on_recovery(&substitute_client_state.sovereign_params);

    let da_params_matches = subject_client_state
        .da_params
        .check_on_recovery(&substitute_client_state.da_params);

    (sov_params_matches && da_params_matches)
        .then_some(())
        .ok_or(ClientError::ClientRecoveryStateMismatch)
}
