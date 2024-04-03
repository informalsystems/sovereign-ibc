use ibc_client_tendermint::context::TmVerifier;
use ibc_client_tendermint::types::Header as TmHeader;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::host::types::identifiers::{ChainId, ClientId};
use ibc_core::host::types::path::ClientConsensusStatePath;
use sov_celestia_client_types::client_message::SovTmHeader;
use sov_celestia_client_types::consensus_state::SovTmConsensusState;
use sov_celestia_client_types::error::IntoResult;
use tendermint_light_client_verifier::options::Options;
use tendermint_light_client_verifier::types::{TrustedBlockState, UntrustedBlockState};
use tendermint_light_client_verifier::Verifier;

use crate::context::{ConsensusStateConverter, ValidationContext as SovValidationContext};

/// Verifies the IBC header type for the Sovereign SDK rollups, which consists
/// of the DA header and the aggregated proof date validation.
pub fn verify_header<V>(
    ctx: &V,
    header: &SovTmHeader,
    client_id: &ClientId,
    chain_id: &ChainId,
    options: &Options,
    verifier: &impl TmVerifier,
) -> Result<(), ClientError>
where
    V: SovValidationContext,
    V::ConsensusStateRef: ConsensusStateConverter,
{
    // Checks the sanity of the fields in the header.
    header.validate_basic()?;

    verify_da_header(
        ctx,
        &header.da_header,
        client_id,
        chain_id,
        options,
        verifier,
    )?;

    // TODO: Implement the verification of the `AggregatedProofData`.
    // aggregated_proof_date.verify()?;

    Ok(())
}

/// Verifies the DA header type for the Sovereign SDK rollups against the
/// trusted state.
pub fn verify_da_header<V>(
    ctx: &V,
    da_header: &TmHeader,
    client_id: &ClientId,
    chain_id: &ChainId,
    options: &Options,
    verifier: &impl TmVerifier,
) -> Result<(), ClientError>
where
    V: SovValidationContext,
    V::ConsensusStateRef: ConsensusStateConverter,
{
    // The revision number of the `ChainId` tracked by the client state must
    // match the `ChainId` in the DA header.
    da_header
        .verify_chain_id_version_matches_height(chain_id)
        .map_err(|e| ClientError::Other {
            description: format!("failed to verify chain id: {e}"),
        })?;

    let trusted_height = da_header.trusted_height;

    let trusted_state = {
        let trusted_client_cons_state_path = ClientConsensusStatePath::new(
            client_id.clone(),
            trusted_height.revision_number(),
            trusted_height.revision_height(),
        );
        let trusted_consensus_state = ctx
            .consensus_state(&trusted_client_cons_state_path)?
            .try_into()?;

        da_header.check_trusted_next_validator_set(
            &trusted_consensus_state.da_params.next_validators_hash,
        )?;

        TrustedBlockState {
            chain_id: &chain_id
                .to_string()
                .try_into()
                .map_err(|e| ClientError::Other {
                    description: format!("failed to parse chain id: {e}"),
                })?,
            header_time: trusted_consensus_state.timestamp(),
            height: trusted_height.revision_height().try_into().map_err(|_| {
                ClientError::Other {
                    description: "failed to convert revision height to u64".to_string(),
                }
            })?,
            next_validators: &da_header.trusted_next_validator_set,
            next_validators_hash: trusted_consensus_state.da_params.next_validators_hash,
        }
    };

    let untrusted_state = UntrustedBlockState {
        signed_header: &da_header.signed_header,
        validators: &da_header.validator_set,
        // NB: This will skip the
        // VerificationPredicates::next_validators_match check for the
        // untrusted state.
        next_validators: None,
    };

    let now = ctx
        .host_timestamp()?
        .into_tm_time()
        .ok_or_else(|| ClientError::ClientSpecific {
            description: "host timestamp is not a valid TM timestamp".to_string(),
        })?;

    // main header verification, delegated to the tendermint-light-client crate.
    verifier
        .verifier()
        .verify_update_header(untrusted_state, trusted_state, options, now)
        .into_result()?;

    Ok(())
}

/// Checks for DA misbehaviour upon receiving a new consensus state as part of a
/// client update.
pub fn check_da_misbehaviour_on_update<V>(
    ctx: &V,
    header: SovTmHeader,
    client_id: &ClientId,
    client_latest_height: &Height,
) -> Result<bool, ClientError>
where
    V: SovValidationContext,
    V::ConsensusStateRef: ConsensusStateConverter,
{
    let maybe_existing_consensus_state = {
        let path_at_header_height = ClientConsensusStatePath::new(
            client_id.clone(),
            header.height().revision_number(),
            header.height().revision_height(),
        );

        ctx.consensus_state(&path_at_header_height).ok()
    };

    if let Some(existing_consensus_state) = maybe_existing_consensus_state {
        let existing_consensus_state = existing_consensus_state.try_into()?;

        let header_consensus_state = SovTmConsensusState::from(header);

        // There is evidence of misbehaviour if the stored consensus state
        // is different from the new one we received.
        Ok(existing_consensus_state != header_consensus_state)
    } else {
        // If no header was previously installed, we ensure the monotonicity of timestamps.

        // 1. for all headers, the new header needs to have a larger timestamp than
        //    the “previous header”
        {
            let maybe_prev_cs = ctx.prev_consensus_state(client_id, &header.height())?;

            if let Some(prev_cs) = maybe_prev_cs {
                // New header timestamp cannot occur *before* the
                // previous consensus state's height
                let prev_cs = prev_cs.try_into()?;

                if header.da_header.signed_header.header().time <= prev_cs.timestamp() {
                    return Ok(true);
                }
            }
        }

        // 2. if a header comes in and is not the “last” header, then we also ensure
        //    that its timestamp is less than the “next header”
        if &header.height() < client_latest_height {
            let maybe_next_cs = ctx.next_consensus_state(client_id, &header.height())?;

            if let Some(next_cs) = maybe_next_cs {
                // New (untrusted) header timestamp cannot occur *after* next
                // consensus state's height
                let next_cs = next_cs.try_into()?;

                if header.da_header.signed_header.header().time >= next_cs.timestamp() {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}
