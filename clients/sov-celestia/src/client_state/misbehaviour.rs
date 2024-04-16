use ibc_client_tendermint::client_state::verify_misbehaviour_header;
use ibc_client_tendermint::verifier::TmVerifier;
use ibc_core::client::context::{Convertible, ExtClientValidationContext};
use ibc_core::client::types::error::ClientError;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::ClientConsensusStatePath;
use sov_celestia_client_types::client_message::SovTmMisbehaviour;
use sov_celestia_client_types::client_state::SovTmClientState;
use sov_celestia_client_types::consensus_state::SovTmConsensusState;
use tendermint::crypto::Sha256;
use tendermint::merkle::MerkleHash;

/// Determines whether or not two conflicting headers at the same height would
/// have convinced the light client.
pub fn verify_misbehaviour<V, H>(
    ctx: &V,
    client_state: &SovTmClientState,
    misbehaviour: &SovTmMisbehaviour,
    client_id: &ClientId,
    verifier: &impl TmVerifier,
) -> Result<(), ClientError>
where
    V: ExtClientValidationContext,
    V::ConsensusStateRef: Convertible<SovTmConsensusState, ClientError>,
    H: MerkleHash + Sha256 + Default,
{
    misbehaviour.validate_basic::<H>()?;

    let header_1 = misbehaviour.header_1();
    let trusted_consensus_state_1 = {
        let consensus_state_path = ClientConsensusStatePath::new(
            client_id.clone(),
            header_1.trusted_height().revision_number(),
            header_1.trusted_height().revision_height(),
        );
        let consensus_state = ctx.consensus_state(&consensus_state_path)?;

        consensus_state.try_into()?
    };

    let header_2 = misbehaviour.header_2();
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

    verify_misbehaviour_header::<H>(
        &header_1.da_header,
        client_state.chain_id(),
        &client_state.as_light_client_options()?,
        trusted_consensus_state_1.timestamp(),
        trusted_consensus_state_1.da_params.next_validators_hash,
        current_timestamp,
        verifier,
    )?;

    verify_misbehaviour_header::<H>(
        &header_2.da_header,
        client_state.chain_id(),
        &client_state.as_light_client_options()?,
        trusted_consensus_state_2.timestamp(),
        trusted_consensus_state_2.da_params.next_validators_hash,
        current_timestamp,
        verifier,
    )?;

    // TODO: Determine what sort of checks we need to carry out for detecting
    // `AggregatedProof` misbehaviour.

    Ok(())
}
