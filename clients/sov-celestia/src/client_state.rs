//! Implements the core [`ClientState`](crate::core::ics02_client::client_state::ClientState) trait
//! for the Sovereign light client.
use alloc::vec::Vec;
use core::convert::{TryFrom, TryInto};

use ibc_client_tendermint::client_state::ClientState as TmClientState;
use ibc_client_tendermint::consensus_state::{ConsensusState as TmConsensusState, ConsensusState};
use ibc_client_tendermint::context::{CommonContext, ValidationContext as TmValidationContext};
use ibc_client_tendermint::types::{
    ClientState as ClientStateType, ConsensusState as ConsensusStateType,
};
use ibc_core::client::context::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation,
};
use ibc_core::client::context::{ClientExecutionContext, ClientValidationContext};
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::{Height, Status, UpdateKind};
use ibc_core::commitment_types::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use ibc_core::host::types::identifiers::{ClientId, ClientType};
use ibc_core::host::types::path::{ClientConsensusStatePath, ClientStatePath, Path};
use ibc_core::host::ExecutionContext;
use ibc_core::primitives::proto::{Any, Protobuf};
use sov_celestia_client_types::client_message::{SovHeader, SovMisbehaviour};
use sov_celestia_client_types::client_state::{
    sov_client_type, RollupClientState, SovTmClientState, SOV_TENDERMINT_CLIENT_STATE_TYPE_URL,
};
use sov_celestia_client_types::proto::SovTmClientState as RawSovTmClientState;

/// Newtype wrapper exists so that we can bypass Rust's orphan rules and
/// implement traits from `ibc::core::client::context` on the `ClientState`
/// type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, derive_more::From)]
pub struct ClientState {
    pub da_client_state: TmClientState,
    pub rollup_client_state: RollupClientState,
}

impl ClientState {
    pub fn new(da_client_state: TmClientState, rollup_client_state: RollupClientState) -> Self {
        Self {
            da_client_state,
            rollup_client_state,
        }
    }
}

impl Protobuf<RawSovTmClientState> for ClientState {}

impl TryFrom<RawSovTmClientState> for ClientState {
    type Error = ClientError;

    fn try_from(raw: RawSovTmClientState) -> Result<Self, Self::Error> {
        let sov_client_state = SovTmClientState::try_from(raw)?;
        Ok(Self {
            da_client_state: sov_client_state.da_client_state.into(),
            rollup_client_state: sov_client_state.rollup_client_state,
        })
    }
}

impl From<ClientState> for RawSovTmClientState {
    fn from(client_state: ClientState) -> Self {
        SovTmClientState {
            da_client_state: client_state.da_client_state.inner().clone(),
            rollup_client_state: client_state.rollup_client_state.clone(),
        }
        .into()
    }
}

impl Protobuf<Any> for ClientState {}

impl TryFrom<Any> for ClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        let any = SovTmClientState::try_from(raw)?;

        Ok(Self {
            da_client_state: any.da_client_state.into(),
            rollup_client_state: any.rollup_client_state,
        })
    }
}

impl From<ClientState> for Any {
    fn from(client_state: ClientState) -> Self {
        Any {
            type_url: SOV_TENDERMINT_CLIENT_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawSovTmClientState>::encode_vec(client_state),
        }
    }
}

impl ClientStateCommon for ClientState {
    fn verify_consensus_state(&self, consensus_state: Any) -> Result<(), ClientError> {
        self.da_client_state.verify_consensus_state(consensus_state)
    }

    fn client_type(&self) -> ClientType {
        sov_client_type()
    }

    fn latest_height(&self) -> Height {
        self.da_client_state.latest_height()
    }

    fn validate_proof_height(&self, proof_height: Height) -> Result<(), ClientError> {
        self.da_client_state.validate_proof_height(proof_height)
    }

    /// Perform client-specific verifications and check all data in the new
    /// client state to be the same across all valid clients for the new chain.
    ///
    /// You can learn more about how to upgrade IBC-connected SDK chains in
    /// [this](https://ibc.cosmos.network/main/ibc/upgrades/quick-guide.html)
    /// guide
    fn verify_upgrade_client(
        &self,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
        proof_upgrade_client: CommitmentProofBytes,
        proof_upgrade_consensus_state: CommitmentProofBytes,
        root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        self.da_client_state.verify_upgrade_client(
            upgraded_client_state,
            upgraded_consensus_state,
            proof_upgrade_client,
            proof_upgrade_consensus_state,
            root,
        )
    }

    fn verify_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
        value: Vec<u8>,
    ) -> Result<(), ClientError> {
        self.da_client_state
            .verify_membership(prefix, proof, root, path, value)
    }

    fn verify_non_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
    ) -> Result<(), ClientError> {
        self.da_client_state
            .verify_non_membership(prefix, proof, root, path)
    }
}

impl<V> ClientStateValidation<V> for ClientState
where
    V: ClientValidationContext + TmValidationContext,
    V::AnyConsensusState: TryInto<ConsensusState>,
    ClientError: From<<V::AnyConsensusState as TryInto<ConsensusState>>::Error>,
{
    fn verify_client_message(
        &self,
        ctx: &V,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        match update_kind {
            UpdateKind::UpdateClient => {
                let header = SovHeader::try_from(client_message)?;
                self.da_client_state
                    .verify_header(ctx, client_id, header.core_header)
            }
            UpdateKind::SubmitMisbehaviour => {
                let misbehaviour = SovMisbehaviour::try_from(client_message)?;
                self.da_client_state.verify_misbehaviour(
                    ctx,
                    client_id,
                    misbehaviour.into_tendermint_misbehaviour(),
                )
            }
        }
    }

    fn check_for_misbehaviour(
        &self,
        ctx: &V,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<bool, ClientError> {
        match update_kind {
            UpdateKind::UpdateClient => {
                let header = SovHeader::try_from(client_message)?;
                self.da_client_state.check_for_misbehaviour_update_client(
                    ctx,
                    client_id,
                    header.core_header,
                )
            }
            UpdateKind::SubmitMisbehaviour => {
                let misbehaviour = SovMisbehaviour::try_from(client_message)?;
                self.da_client_state.check_for_misbehaviour_misbehavior(
                    &misbehaviour.into_tendermint_misbehaviour(),
                )
            }
        }
    }

    fn status(&self, ctx: &V, client_id: &ClientId) -> Result<Status, ClientError> {
        self.da_client_state.status(ctx, client_id)
    }
}

impl<E> ClientStateExecution<E> for ClientState
where
    E: ClientExecutionContext + TmValidationContext + ExecutionContext,
    <E as ClientExecutionContext>::AnyClientState: From<ClientState>,
    <E as ClientExecutionContext>::AnyClientState: From<TmClientState>,
    <E as ClientExecutionContext>::AnyConsensusState: From<ConsensusState>,
{
    fn initialise(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        consensus_state: Any,
    ) -> Result<(), ClientError> {
        let host_timestamp = CommonContext::host_timestamp(ctx)?;
        let host_height = CommonContext::host_height(ctx)?;

        let tm_consensus_state = ConsensusState::try_from(consensus_state)?;

        ctx.store_client_state(ClientStatePath::new(client_id), self.clone().into())?;
        ctx.store_consensus_state(
            ClientConsensusStatePath::new(
                client_id.clone(),
                self.da_client_state.latest_height().revision_number(),
                self.da_client_state.latest_height().revision_height(),
            ),
            tm_consensus_state.into(),
        )?;
        ctx.store_update_time(client_id.clone(), self.latest_height(), host_timestamp)?;
        ctx.store_update_height(client_id.clone(), self.latest_height(), host_height)?;

        Ok(())
    }

    fn update_state(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        header: Any,
    ) -> Result<Vec<Height>, ClientError> {
        let header = SovHeader::try_from(header)?;
        let header_height = header.height();

        self.da_client_state
            .prune_oldest_consensus_state(ctx, client_id)?;

        let maybe_existing_consensus_state = {
            let path_at_header_height = ClientConsensusStatePath::new(
                client_id.clone(),
                header_height.revision_number(),
                header_height.revision_height(),
            );

            CommonContext::consensus_state(ctx, &path_at_header_height).ok()
        };

        if maybe_existing_consensus_state.is_some() {
            // if we already had the header installed by a previous relayer
            // then this is a no-op.
            //
            // Do nothing.
        } else {
            let host_timestamp = CommonContext::host_timestamp(ctx)?;
            let host_height = CommonContext::host_height(ctx)?;

            let new_consensus_state = ConsensusStateType::from(header.core_header.clone());
            let new_da_client_state = self
                .da_client_state
                .inner()
                .clone()
                .with_header(header.core_header)?;

            let new_client_state = ClientState::new(
                new_da_client_state.clone().into(),
                self.rollup_client_state.clone(),
            );

            ctx.store_consensus_state(
                ClientConsensusStatePath::new(
                    client_id.clone(),
                    new_da_client_state.latest_height.revision_number(),
                    new_da_client_state.latest_height.revision_height(),
                ),
                TmConsensusState::from(new_consensus_state).into(),
            )?;
            ctx.store_client_state(ClientStatePath::new(client_id), new_client_state.into())?;
            ctx.store_update_time(client_id.clone(), header_height, host_timestamp)?;
            ctx.store_update_height(client_id.clone(), header_height, host_height)?;
        }

        Ok(vec![header_height])
    }

    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        _client_message: Any,
        _update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        let frozen_client_state = self
            .da_client_state
            .inner()
            .clone()
            .with_frozen_height(Height::min(0));

        let frozen_da_client_state = TmClientState::from(frozen_client_state);

        let frozen_client_state = ClientState::new(
            frozen_da_client_state.clone(),
            self.rollup_client_state.clone(),
        );

        ctx.store_client_state(ClientStatePath::new(client_id), frozen_client_state.into())?;

        Ok(())
    }

    // Commit the new client state and consensus state to the store
    fn update_state_on_upgrade(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
    ) -> Result<Height, ClientError> {
        let upgraded_tm_client_state = Self::try_from(upgraded_client_state)?;
        let upgraded_tm_cons_state = TmConsensusState::try_from(upgraded_consensus_state)?;

        let mut upgraded_da_client_state = upgraded_tm_client_state.da_client_state.inner().clone();

        let tm_client_state = self.da_client_state.inner();

        upgraded_da_client_state.zero_custom_fields();

        // Construct new client state and consensus state relayer chosen client
        // parameters are ignored. All chain-chosen parameters come from
        // committed client, all client-chosen parameters come from current
        // client.
        let new_da_client_state = ClientStateType::new(
            upgraded_da_client_state.chain_id,
            tm_client_state.trust_level,
            tm_client_state.trusting_period,
            upgraded_da_client_state.unbonding_period,
            tm_client_state.max_clock_drift,
            upgraded_da_client_state.latest_height,
            upgraded_da_client_state.proof_specs,
            upgraded_da_client_state.upgrade_path,
            tm_client_state.allow_update,
        )?;

        let new_client_state = ClientState::new(
            new_da_client_state.clone().into(),
            self.rollup_client_state.clone(),
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
        let new_consensus_state = ConsensusStateType::new(
            sentinel_root.into(),
            upgraded_tm_cons_state.timestamp(),
            upgraded_tm_cons_state.next_validators_hash(),
        );

        let latest_height = new_da_client_state.latest_height;
        let host_timestamp = CommonContext::host_timestamp(ctx)?;
        let host_height = CommonContext::host_height(ctx)?;

        ctx.store_client_state(ClientStatePath::new(client_id), new_client_state.into())?;
        ctx.store_consensus_state(
            ClientConsensusStatePath::new(
                client_id.clone(),
                latest_height.revision_number(),
                latest_height.revision_height(),
            ),
            TmConsensusState::from(new_consensus_state).into(),
        )?;
        ctx.store_update_time(client_id.clone(), latest_height, host_timestamp)?;
        ctx.store_update_height(client_id.clone(), latest_height, host_height)?;

        Ok(latest_height)
    }
}
