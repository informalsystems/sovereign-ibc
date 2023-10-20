use alloc::format;

use ibc::core::ics02_client::error::ClientError;
use ibc::core::ics24_host::identifier::ClientId;
use ibc::core::ics24_host::path::ClientConsensusStatePath;
use tendermint::block::signed_header::SignedHeader;
use tendermint_light_client_verifier::types::{TrustedBlockState, UntrustedBlockState};

use super::SovClientState;
use crate::context::custom_ctx::ValidationContext as SovValidationContext;
use crate::types::client_message::SovHeader;
use crate::types::consensus_state::SovConsensusState;

impl SovClientState {
    pub fn verify_header<ClientValidationContext>(
        &self,
        ctx: &ClientValidationContext,
        client_id: &ClientId,
        header: SovHeader,
    ) -> Result<(), ClientError>
    where
        ClientValidationContext: SovValidationContext,
    {
        // Checks that the header fields are valid.
        header.validate_basic()?;

        // The tendermint-light-client crate though works on heights that are assumed
        // to have the same revision number. We ensure this here.
        header.verify_chain_id_version_matches_height(&self.chain_id())?;

        // Delegate to tendermint-light-client, which contains the required checks
        // of the new header against the trusted consensus state.
        {
            let _trusted_state = {
                let trusted_client_cons_state_path =
                    ClientConsensusStatePath::new(client_id, &header.da_header.trusted_height);
                let trusted_consensus_state: SovConsensusState = ctx
                    .consensus_state(&trusted_client_cons_state_path)?
                    .try_into()
                    .map_err(|err| ClientError::Other {
                        description: err.to_string(),
                    })?;

                check_header_trusted_next_validator_set(&header, &trusted_consensus_state)?;

                TrustedBlockState {
                    chain_id: &self.chain_id.to_string().try_into().map_err(|e| {
                        ClientError::Other {
                            description: format!("failed to parse chain id: {}", e),
                        }
                    })?,
                    header_time: trusted_consensus_state.timestamp,
                    height: header
                        .da_header
                        .trusted_height
                        .revision_height()
                        .try_into()
                        .map_err(|_| ClientError::ClientSpecific {
                            description: "header trusted height is not a valid TM height"
                                .to_string(),
                        })?,
                    next_validators: &header.da_header.trusted_next_validator_set,
                    next_validators_hash: trusted_consensus_state.next_validators_hash,
                }
            };

            let _untrusted_state = UntrustedBlockState {
                signed_header: &SignedHeader::new(
                    header.da_header.extended_header.header.clone(),
                    header.da_header.extended_header.commit.clone(),
                )
                .unwrap(),
                validators: &header.da_header.extended_header.validator_set,
                // NB: This will skip the
                // VerificationPredicates::next_validators_match check for the
                // untrusted state.
                next_validators: None,
            };

            let _options = self.as_light_client_options()?;
            let _now = ctx.host_timestamp()?.into_tm_time().ok_or_else(|| {
                ClientError::ClientSpecific {
                    description: "host timestamp is not a valid TM timestamp".to_string(),
                }
            })?;

            // // main header verification, delegated to the tendermint-light-client crate.
            // self.verifier
            //     .verify_update_header(untrusted_state, trusted_state, &options, now)
            //     .into_result()?;
        }

        Ok(())
    }

    pub fn check_for_misbehaviour_update_client<ClientValidationContext>(
        &self,
        ctx: &ClientValidationContext,
        client_id: &ClientId,
        header: SovHeader,
    ) -> Result<bool, ClientError>
    where
        ClientValidationContext: SovValidationContext,
    {
        let header_consensus_state = SovConsensusState::from(header.clone());

        let maybe_existing_consensus_state = {
            let path_at_header_height = ClientConsensusStatePath::new(client_id, &header.height());

            ctx.consensus_state(&path_at_header_height).ok()
        };

        match maybe_existing_consensus_state {
            Some(existing_consensus_state) => {
                let existing_consensus_state: SovConsensusState = existing_consensus_state
                    .try_into()
                    .map_err(|err| ClientError::Other {
                        description: err.to_string(),
                    })?;

                // There is evidence of misbehaviour if the stored consensus state
                // is different from the new one we received.
                Ok(existing_consensus_state != header_consensus_state)
            }
            None => {
                // If no header was previously installed, we ensure the monotonicity of timestamps.

                // 1. for all headers, the new header needs to have a larger timestamp than
                //    the “previous header”
                {
                    let maybe_prev_cs = ctx.prev_consensus_state(client_id, &header.height())?;

                    if let Some(prev_cs) = maybe_prev_cs {
                        // New header timestamp cannot occur *before* the
                        // previous consensus state's height
                        let prev_cs: SovConsensusState =
                            prev_cs.try_into().map_err(|err| ClientError::Other {
                                description: err.to_string(),
                            })?;

                        if header.da_header.extended_header.header.time <= prev_cs.timestamp {
                            return Ok(true);
                        }
                    }
                }

                // 2. if a header comes in and is not the “last” header, then we also ensure
                //    that its timestamp is less than the “next header”
                if header.height() < self.latest_height {
                    let maybe_next_cs = ctx.next_consensus_state(client_id, &header.height())?;

                    if let Some(next_cs) = maybe_next_cs {
                        // New (untrusted) header timestamp cannot occur *after* next
                        // consensus state's height
                        let next_cs: SovConsensusState =
                            next_cs.try_into().map_err(|err| ClientError::Other {
                                description: err.to_string(),
                            })?;

                        if header.da_header.extended_header.header.time >= next_cs.timestamp {
                            return Ok(true);
                        }
                    }
                }

                Ok(false)
            }
        }
    }
}

// `header.trusted_validator_set` was given to us by the relayer. Thus, we
// need to ensure that the relayer gave us the right set, i.e. by ensuring
// that it matches the hash we have stored on chain.
pub(crate) fn check_header_trusted_next_validator_set(
    header: &SovHeader,
    trusted_consensus_state: &SovConsensusState,
) -> Result<(), ClientError> {
    if header.da_header.trusted_next_validator_set.hash()
        == trusted_consensus_state.next_validators_hash
    {
        Ok(())
    } else {
        Err(ClientError::HeaderVerificationFailure {
            reason: "header trusted next validator set hash does not match hash stored on chain"
                .to_string(),
        })
    }
}
