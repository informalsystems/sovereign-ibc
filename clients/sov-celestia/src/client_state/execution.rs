use alloc::vec::Vec;

use ibc_core::client::context::client_state::ClientStateExecution;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::{ClientConsensusStatePath, ClientStatePath};
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::proto::Any;
use sov_celestia_client_types::client_message::Header;
use sov_celestia_client_types::client_state::{SovTmClientState, TmClientParams};
use sov_celestia_client_types::consensus_state::{
    ConsensusState as ConsensusStateType, SovTmConsensusState, TmConsensusParams,
};

use super::ClientState;
use crate::consensus_state::ConsensusState;
use crate::context::{
    ConsensusStateConverter, ExecutionContext as SovExecutionContext,
    ValidationContext as SovValidationContext,
};

impl<E> ClientStateExecution<E> for ClientState
where
    E: SovExecutionContext,
    E::ClientStateRef: From<SovTmClientState>,
    E::ConsensusStateRef: ConsensusStateConverter,
{
    fn initialise(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        consensus_state: Any,
    ) -> Result<(), ClientError> {
        initialise(self.inner(), ctx, client_id, consensus_state)
    }

    fn update_state(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        header: Any,
    ) -> Result<Vec<Height>, ClientError> {
        update_state(self.inner(), ctx, client_id, header)
    }

    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        _client_message: Any,
    ) -> Result<(), ClientError> {
        update_state_on_misbehaviour(self.inner(), ctx, client_id, _client_message)
    }

    // Commit the new client state and consensus state to the store
    fn update_state_on_upgrade(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
    ) -> Result<Height, ClientError> {
        update_state_on_upgrade(
            self.inner(),
            ctx,
            client_id,
            upgraded_client_state,
            upgraded_consensus_state,
        )
    }

    fn update_on_recovery(
        &self,
        ctx: &mut E,
        subject_client_id: &ClientId,
        substitute_client_state: Any,
    ) -> Result<(), ClientError> {
        update_on_recovery(
            self.inner(),
            ctx,
            subject_client_id,
            substitute_client_state,
        )
    }
}

pub fn initialise<E>(
    client_state: &SovTmClientState,
    ctx: &mut E,
    client_id: &ClientId,
    consensus_state: Any,
) -> Result<(), ClientError>
where
    E: SovExecutionContext,
    E::ClientStateRef: From<SovTmClientState>,
    E::ConsensusStateRef: ConsensusStateConverter,
{
    let host_timestamp = SovValidationContext::host_timestamp(ctx)?;
    let host_height = SovValidationContext::host_height(ctx)?;

    let sov_consensus_state = SovTmConsensusState::try_from(consensus_state)?;

    ctx.store_client_state(
        ClientStatePath::new(client_id.clone()),
        client_state.clone().into(),
    )?;
    ctx.store_consensus_state(
        ClientConsensusStatePath::new(
            client_id.clone(),
            client_state.latest_height().revision_number(),
            client_state.latest_height().revision_height(),
        ),
        sov_consensus_state.into(),
    )?;
    ctx.store_update_meta(
        client_id.clone(),
        client_state.latest_height(),
        host_timestamp,
        host_height,
    )?;

    Ok(())
}

pub fn update_state<E>(
    client_state: &SovTmClientState,
    ctx: &mut E,
    client_id: &ClientId,
    header: Any,
) -> Result<Vec<Height>, ClientError>
where
    E: SovExecutionContext,
    E::ClientStateRef: From<SovTmClientState>,
    E::ConsensusStateRef: ConsensusStateConverter,
{
    let header = Header::try_from(header)?;
    let header_height = header.height();

    // self.prune_oldest_consensus_state(ctx, client_id)?;

    let maybe_existing_consensus_state = {
        let path_at_header_height = ClientConsensusStatePath::new(
            client_id.clone(),
            header_height.revision_number(),
            header_height.revision_height(),
        );

        ctx.consensus_state(&path_at_header_height).ok()
    };

    if maybe_existing_consensus_state.is_some() {
        // if we already had the header installed by a previous relayer
        // then this is a no-op.
        //
        // Do nothing.
    } else {
        let host_timestamp = SovValidationContext::host_timestamp(ctx)?;
        let host_height = SovValidationContext::host_height(ctx)?;

        let new_consensus_state = ConsensusStateType::from(header.clone());
        let new_client_state = client_state.clone().with_header(header.da_header)?;

        ctx.store_consensus_state(
            ClientConsensusStatePath::new(
                client_id.clone(),
                new_client_state.latest_height.revision_number(),
                new_client_state.latest_height.revision_height(),
            ),
            new_consensus_state.into(),
        )?;
        ctx.store_client_state(
            ClientStatePath::new(client_id.clone()),
            new_client_state.into(),
        )?;
        ctx.store_update_meta(
            client_id.clone(),
            header_height,
            host_timestamp,
            host_height,
        )?;
    }

    Ok(vec![header_height])
}

pub fn update_state_on_misbehaviour<E>(
    client_state: &SovTmClientState,
    ctx: &mut E,
    client_id: &ClientId,
    _client_message: Any,
) -> Result<(), ClientError>
where
    E: SovExecutionContext,
    E::ClientStateRef: From<SovTmClientState>,
    E::ConsensusStateRef: ConsensusStateConverter,
{
    let frozen_client_state = client_state.clone().with_frozen_height(Height::min(0));

    ctx.store_client_state(
        ClientStatePath::new(client_id.clone()),
        frozen_client_state.into(),
    )?;

    Ok(())
}

pub fn update_state_on_upgrade<E>(
    _client_state: &SovTmClientState,
    ctx: &mut E,
    client_id: &ClientId,
    upgraded_client_state: Any,
    upgraded_consensus_state: Any,
) -> Result<Height, ClientError>
where
    E: SovExecutionContext,
    E::ClientStateRef: From<SovTmClientState>,
    E::ConsensusStateRef: ConsensusStateConverter,
{
    let mut upgraded_client_state = SovTmClientState::try_from(upgraded_client_state)?;
    let upgraded_tm_cons_state = ConsensusState::try_from(upgraded_consensus_state)?;

    upgraded_client_state.zero_custom_fields();

    // Construct new client state and consensus state relayer chosen client
    // parameters are ignored. All chain-chosen parameters come from
    // committed client, all client-chosen parameters come from current
    // client.
    let new_client_state = SovTmClientState::new(
        upgraded_client_state.rollup_id,
        upgraded_client_state.latest_height,
        None,
        upgraded_client_state.upgrade_path,
        upgraded_client_state.da_params,
    );

    // The new consensus state is merely used as a trusted kernel against
    // which headers on the new chain can be verified. The root is just a
    // stand-in sentinel value as it cannot be known in advance, thus no
    // proof verification will pass. The timestamp and the
    // NextValidatorsHash of the consensus state is the blocktime and
    // NextValidatorsHash of the last block committed by the old chain. This
    // will allow the first block of the new chain to be verified against
    // the last validators of the old chain so long as it is submitted
    // within the TrustingPeriod of this client.
    // NOTE: We do not set processed time for this consensus state since
    // this consensus state should not be used for packet verification as
    // the root is empty. The next consensus state submitted using update
    // will be usable for packet-verification.
    let sentinel_root = "sentinel_root".as_bytes().to_vec();
    let new_consensus_state = SovTmConsensusState::new(
        sentinel_root.into(),
        TmConsensusParams {
            timestamp: upgraded_tm_cons_state.timestamp(),
            next_validators_hash: upgraded_tm_cons_state.next_validators_hash(),
        },
    );

    let latest_height = new_client_state.latest_height;
    let host_timestamp = SovValidationContext::host_timestamp(ctx)?;
    let host_height = SovValidationContext::host_height(ctx)?;

    ctx.store_client_state(
        ClientStatePath::new(client_id.clone()),
        new_client_state.into(),
    )?;
    ctx.store_consensus_state(
        ClientConsensusStatePath::new(
            client_id.clone(),
            latest_height.revision_number(),
            latest_height.revision_height(),
        ),
        new_consensus_state.into(),
    )?;
    ctx.store_update_meta(
        client_id.clone(),
        latest_height,
        host_timestamp,
        host_height,
    )?;

    Ok(latest_height)
}

/// Update the client's chain ID, trusting period, latest height, processed
/// height, and processed time metadata values to those values provided by a
/// verified substitute client state in response to a successful client
/// recovery.
pub fn update_on_recovery<E>(
    subject_client_state: &SovTmClientState,
    ctx: &mut E,
    subject_client_id: &ClientId,
    substitute_client_state: Any,
) -> Result<(), ClientError>
where
    E: SovExecutionContext,
    E::ClientStateRef: From<SovTmClientState>,
    E::ConsensusStateRef: ConsensusStateConverter,
{
    let substitute_client_state = ClientState::try_from(substitute_client_state)?.into_inner();

    let chain_id = substitute_client_state.da_params.chain_id;
    let trusting_period = substitute_client_state.da_params.trusting_period;
    let latest_height = substitute_client_state.latest_height;

    let new_client_state = SovTmClientState {
        rollup_id: subject_client_state.rollup_id.clone(),
        latest_height,
        frozen_height: None,
        upgrade_path: subject_client_state.upgrade_path.clone(),
        da_params: TmClientParams {
            chain_id,
            trusting_period,
            ..subject_client_state.da_params
        },
    };

    let host_timestamp = E::host_timestamp(ctx)?;
    let host_height = E::host_height(ctx)?;

    ctx.store_client_state(
        ClientStatePath::new(subject_client_id.clone()),
        new_client_state.into(),
    )?;

    ctx.store_update_meta(
        subject_client_id.clone(),
        latest_height,
        host_timestamp,
        host_height,
    )?;

    Ok(())
}
