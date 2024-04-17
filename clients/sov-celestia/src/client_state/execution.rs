use ibc_core::client::context::client_state::ClientStateExecution;
use ibc_core::client::context::{
    Convertible, ExtClientExecutionContext, ExtClientValidationContext,
};
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::{ClientConsensusStatePath, ClientStatePath};
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::proto::Any;
use sov_celestia_client_types::client_message::Header;
use sov_celestia_client_types::client_state::SovTmClientState;
use sov_celestia_client_types::consensus_state::{
    ConsensusState as ConsensusStateType, SovTmConsensusState, TmConsensusParams,
};
use sov_celestia_client_types::sovereign::SovereignConsensusParams;

use super::ClientState;
use crate::consensus_state::ConsensusState;

impl<E> ClientStateExecution<E> for ClientState
where
    E: ExtClientExecutionContext,
    E::ClientStateRef: From<SovTmClientState>,
    E::ConsensusStateRef: Convertible<SovTmConsensusState, ClientError>,
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
            self.inner().clone(),
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
            self.inner().clone(),
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
    E: ExtClientExecutionContext,
    E::ClientStateRef: From<SovTmClientState>,
    E::ConsensusStateRef: Convertible<SovTmConsensusState, ClientError>,
{
    let host_timestamp = ExtClientValidationContext::host_timestamp(ctx)?;
    let host_height = ExtClientValidationContext::host_height(ctx)?;

    let sov_consensus_state = SovTmConsensusState::try_from(consensus_state)?;

    let latest_height = client_state.latest_height_in_sov();

    ctx.store_client_state(
        ClientStatePath::new(client_id.clone()),
        client_state.clone().into(),
    )?;
    ctx.store_consensus_state(
        ClientConsensusStatePath::new(
            client_id.clone(),
            latest_height.revision_number(),
            latest_height.revision_height(),
        ),
        sov_consensus_state.into(),
    )?;
    ctx.store_update_meta(
        client_id.clone(),
        latest_height,
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
    E: ExtClientExecutionContext,
    E::ClientStateRef: From<SovTmClientState>,
    E::ConsensusStateRef: Convertible<SovTmConsensusState, ClientError>,
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
        let host_timestamp = ExtClientValidationContext::host_timestamp(ctx)?;
        let host_height = ExtClientValidationContext::host_height(ctx)?;

        let new_consensus_state = ConsensusStateType::from(header.clone());
        let new_client_state = client_state.clone().with_da_header(header.da_header)?;

        ctx.store_consensus_state(
            ClientConsensusStatePath::new(
                client_id.clone(),
                header_height.revision_number(),
                header_height.revision_height(),
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
    E: ExtClientExecutionContext,
    E::ClientStateRef: From<SovTmClientState>,
    E::ConsensusStateRef: Convertible<SovTmConsensusState, ClientError>,
{
    let frozen_client_state = client_state.clone().with_frozen_height(Height::min(0));

    ctx.store_client_state(
        ClientStatePath::new(client_id.clone()),
        frozen_client_state.into(),
    )?;

    Ok(())
}

pub fn update_state_on_upgrade<E>(
    client_state: SovTmClientState,
    ctx: &mut E,
    client_id: &ClientId,
    upgraded_client_state: Any,
    upgraded_consensus_state: Any,
) -> Result<Height, ClientError>
where
    E: ExtClientExecutionContext,
    E::ClientStateRef: From<SovTmClientState>,
    E::ConsensusStateRef: Convertible<SovTmConsensusState, ClientError>,
{
    let mut upgraded_client_state = SovTmClientState::try_from(upgraded_client_state)?;
    let upgraded_consensus_state = ConsensusState::try_from(upgraded_consensus_state)?;

    upgraded_client_state.zero_custom_fields();

    // Creates new Sovereign client parameters. The `genesis_da_height` and
    // `genesis_state_root` are considered immutable properties of the client.
    // Changing them implies creating a client that could potentially be compatible
    // with other rollups.
    let new_sovereign_params = client_state
        .sovereign_params
        .update_on_upgrade(upgraded_client_state.sovereign_params);

    // Creates new Tendermint client parameters. All chain-chosen parameters
    // come from committed client, all relayer-chosen parameters come from
    // current client.
    let new_tendermint_params = client_state
        .da_params
        .update_on_upgrade(upgraded_client_state.da_params);

    let new_client_state = SovTmClientState::new(new_sovereign_params, new_tendermint_params);

    // The new consensus state is merely used as a trusted kernel against which
    // headers on the new rollup can be verified. The root is just a stand-in
    // sentinel value as it cannot be known in advance, thus no proof
    // verification will pass. The next consensus state submitted using update
    // will be usable for packet-verification.
    let sentinel_root = "sentinel_root".as_bytes().to_vec();

    let new_sovereign_consensus_params =
        SovereignConsensusParams::new(sentinel_root.clone().into());

    // The `timestamp` and the `next_validators_hash` of the consensus state is
    // the block time and `next_validators_hash` of the last block committed by
    // the old chain. This will allow the first block of the new chain to be
    // verified against the last validators of the old chain so long as it is
    // submitted within the DA `trusting_period` of this client.
    let new_tm_consensus_params = TmConsensusParams {
        timestamp: upgraded_consensus_state.timestamp(),
        next_validators_hash: upgraded_consensus_state.next_validators_hash(),
    };

    let new_consensus_state =
        SovTmConsensusState::new(new_sovereign_consensus_params, new_tm_consensus_params);

    let latest_height = new_client_state.latest_height_in_sov();
    let host_timestamp = ExtClientValidationContext::host_timestamp(ctx)?;
    let host_height = ExtClientValidationContext::host_height(ctx)?;

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
    subject_client_state: SovTmClientState,
    ctx: &mut E,
    subject_client_id: &ClientId,
    substitute_client_state: Any,
) -> Result<(), ClientError>
where
    E: ExtClientExecutionContext,
    E::ClientStateRef: From<SovTmClientState>,
    E::ConsensusStateRef: Convertible<SovTmConsensusState, ClientError>,
{
    let substitute_client_state = ClientState::try_from(substitute_client_state)?.into_inner();

    let latest_height = substitute_client_state.latest_height_in_sov();

    let new_sovereign_params = subject_client_state
        .sovereign_params
        .update_on_recovery(substitute_client_state.sovereign_params);

    let new_da_params = subject_client_state
        .da_params
        .update_on_recovery(substitute_client_state.da_params);

    let new_client_state = SovTmClientState::new(new_sovereign_params, new_da_params);

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
