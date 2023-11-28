use alloc::format;

use ibc_core::client::context::consensus_state::ConsensusState;
use ibc_core::client::types::error::ClientError;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::ClientConsensusStatePath;
use ibc_core::primitives::Timestamp;

use super::update::check_header_trusted_next_validator_set;
use super::SovClientState;
use crate::client_message::{SovHeader, SovMisbehaviour};
use crate::consensus_state::SovConsensusState;
use crate::context::ValidationContext as SovValidationContext;

impl SovClientState {
    // verify_misbehaviour determines whether or not two conflicting headers at
    // the same height would have convinced the light client.
    pub fn verify_misbehaviour<ClientValidationContext>(
        &self,
        ctx: &ClientValidationContext,
        client_id: &ClientId,
        misbehaviour: SovMisbehaviour,
    ) -> Result<(), ClientError>
    where
        ClientValidationContext: SovValidationContext,
    {
        misbehaviour
            .validate_basic()
            .map_err(|e| ClientError::Other {
                description: e.to_string(),
            })?;

        let header_1 = misbehaviour.header1();
        let trusted_consensus_state_1 = {
            let consensus_state_path = ClientConsensusStatePath::new(
                client_id.clone(),
                header_1.da_header.trusted_height.revision_number(),
                header_1.da_header.trusted_height.revision_height(),
            );
            let consensus_state = ctx.consensus_state(&consensus_state_path)?;

            consensus_state
                .try_into()
                .map_err(|err| ClientError::Other {
                    description: err.to_string(),
                })?
        };

        let header_2 = misbehaviour.header2();
        let trusted_consensus_state_2 = {
            let consensus_state_path = ClientConsensusStatePath::new(
                client_id.clone(),
                header_2.da_header.trusted_height.revision_number(),
                header_2.da_header.trusted_height.revision_height(),
            );
            let consensus_state = ctx.consensus_state(&consensus_state_path)?;

            consensus_state
                .try_into()
                .map_err(|err| ClientError::Other {
                    description: err.to_string(),
                })?
        };

        let current_timestamp = ctx.host_timestamp()?;
        self.verify_misbehaviour_header(header_1, &trusted_consensus_state_1, current_timestamp)?;
        self.verify_misbehaviour_header(header_2, &trusted_consensus_state_2, current_timestamp)
    }

    pub fn verify_misbehaviour_header(
        &self,
        header: &SovHeader,
        trusted_consensus_state: &SovConsensusState,
        current_timestamp: Timestamp,
    ) -> Result<(), ClientError> {
        // ensure correctness of the trusted next validator set provided by the relayer
        check_header_trusted_next_validator_set(header, trusted_consensus_state)?;

        // ensure trusted consensus state is within trusting period
        {
            let duration_since_consensus_state = current_timestamp
                .duration_since(&trusted_consensus_state.timestamp())
                .ok_or_else(|| ClientError::InvalidConsensusStateTimestamp {
                    time1: trusted_consensus_state.timestamp(),
                    time2: current_timestamp,
                })?;

            if duration_since_consensus_state >= self.trusting_period {
                return Err(ClientError::Other {
                    description: format!(
                        "misbehaviour header's trusted consensus state is not within trusting period (duration_since_consensus_state: {:?}, trusting_period: {:?})",
                        duration_since_consensus_state, self.trusting_period
                    ),
                });
            }
        }

        // // main header verification, delegated to the sovereign-light-client crate.
        // let untrusted_state = header
        //     .da_header
        //     .extended_header
        //     .header
        //     .as_untrusted_block_state();

        // let chain_id = self
        //     .chain_id
        //     .to_string()
        //     .try_into()
        //     .map_err(|e| ClientError::Other {
        //         description: format!("failed to parse chain id: {}", e),
        //     })?;
        // let trusted_state = header
        //     .da_header
        //     .extended_header
        //     .header
        //     .as_trusted_block_state(trusted_consensus_state, &chain_id)?;

        // let options = self.as_light_client_options()?;
        // let current_timestamp = current_timestamp.into_tm_time().ok_or(ClientError::Other {
        //     description: "host timestamp must not be zero".to_string(),
        // })?;

        // self.verifier
        //     .verify_misbehaviour_header(untrusted_state, trusted_state, &options, current_timestamp)
        //     .into_result()?;

        Ok(())
    }

    pub fn check_for_misbehaviour_misbehavior(
        &self,
        misbehaviour: &SovMisbehaviour,
    ) -> Result<bool, ClientError> {
        let header_1 = misbehaviour.header1();
        let header_2 = misbehaviour.header2();

        if header_1.height() == header_2.height() {
            // when the height of the 2 headers are equal, we only have evidence
            // of misbehaviour in the case where the headers are different
            // (otherwise, the same header was added twice in the message,
            // and this is evidence of nothing)
            Ok(header_1.da_header.extended_header.commit.block_id.hash
                != header_2.da_header.extended_header.commit.block_id.hash)
        } else {
            // header_1 is at greater height than header_2, therefore
            // header_1 time must be less than or equal to
            // header_2 time in order to be valid misbehaviour (violation of
            // monotonic time).
            Ok(header_1.da_header.extended_header.header.time
                <= header_2.da_header.extended_header.header.time)
        }
    }
}
