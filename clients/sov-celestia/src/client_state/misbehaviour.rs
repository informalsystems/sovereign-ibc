use ibc_client_tendermint::client_state::verify_misbehaviour_header;
use ibc_client_tendermint::context::TmVerifier;
use ibc_core::client::types::error::ClientError;
use ibc_core::host::types::identifiers::{ChainId, ClientId};
use ibc_core::host::types::path::ClientConsensusStatePath;
use sov_celestia_client_types::client_message::SovTmMisbehaviour;
use tendermint_light_client_verifier::options::Options;

use crate::context::{ConsensusStateConverter, ValidationContext as SovValidationContext};

/// Determines whether or not two conflicting headers at the same height would
/// have convinced the light client.
pub fn verify_misbehaviour<V>(
    ctx: &V,
    misbehaviour: &SovTmMisbehaviour,
    client_id: &ClientId,
    chain_id: &ChainId,
    options: &Options,
    verifier: &impl TmVerifier,
) -> Result<(), ClientError>
where
    V: SovValidationContext,
    V::ConsensusStateRef: ConsensusStateConverter,
{
    misbehaviour.validate_basic()?;

    let header_1 = misbehaviour.header1();
    let trusted_consensus_state_1 = {
        let consensus_state_path = ClientConsensusStatePath::new(
            client_id.clone(),
            header_1.trusted_height().revision_number(),
            header_1.trusted_height().revision_height(),
        );
        let consensus_state = ctx.consensus_state(&consensus_state_path)?;

        consensus_state.try_into()?
    };

    let header_2 = misbehaviour.header2();
    let trusted_consensus_state_2 = {
        let consensus_state_path = ClientConsensusStatePath::new(
            client_id.clone(),
            header_2.trusted_height().revision_number(),
            header_2.trusted_height().revision_height(),
        );
        let consensus_state = ctx.consensus_state(&consensus_state_path)?;

        consensus_state.try_into()?
    };

    let current_timestamp = ctx.host_timestamp()?;

    verify_misbehaviour_header(
        &header_1.da_header,
        chain_id,
        options,
        trusted_consensus_state_1.timestamp(),
        trusted_consensus_state_1.da_params.next_validators_hash,
        current_timestamp,
        verifier,
    )?;

    verify_misbehaviour_header(
        &header_2.da_header,
        chain_id,
        options,
        trusted_consensus_state_2.timestamp(),
        trusted_consensus_state_2.da_params.next_validators_hash,
        current_timestamp,
        verifier,
    )?;

    // TODO: Determine what sort of checks we need to carry out for detecting
    // `AggregatedProof` misbehaviour.

    Ok(())
}
