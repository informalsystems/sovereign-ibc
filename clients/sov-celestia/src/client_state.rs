//! Implements the core [`ClientState`](ibc_core::client::context::client_state::ClientState) trait
//! for the Sovereign light client.
use alloc::vec::Vec;
use core::convert::{TryFrom, TryInto};

use ibc_core::client::context::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation,
};
use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
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
use sov_celestia_client_types::client_message::Header;
use sov_celestia_client_types::client_state::{
    sov_client_type, ClientState as ClientStateType, SOV_TENDERMINT_CLIENT_STATE_TYPE_URL,
};
use sov_celestia_client_types::consensus_state::ConsensusState as ConsensusStateType;
use sov_celestia_client_types::proto::v1::ClientState as RawSovTmClientState;

use crate::consensus_state::ConsensusState;
use crate::context::{CommonContext, ValidationContext as SovValidationContext};

/// Newtype wrapper exists so that we can bypass Rust's orphan rules and
/// implement traits from `ibc::core::client::context` on the `ClientState`
/// type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, derive_more::From)]
pub struct ClientState(ClientStateType);

impl ClientState {
    pub fn inner(&self) -> &ClientStateType {
        &self.0
    }
}

impl Protobuf<RawSovTmClientState> for ClientState {}

impl TryFrom<RawSovTmClientState> for ClientState {
    type Error = ClientError;

    fn try_from(raw: RawSovTmClientState) -> Result<Self, Self::Error> {
        let sov_client_state = ClientStateType::try_from(raw)?;

        Ok(Self(sov_client_state))
    }
}

impl From<ClientState> for RawSovTmClientState {
    fn from(client_state: ClientState) -> Self {
        client_state.0.into()
    }
}

impl Protobuf<Any> for ClientState {}

impl TryFrom<Any> for ClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        let any = ClientStateType::try_from(raw)?;

        Ok(Self(any))
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
        let tm_consensus_state = ConsensusState::try_from(consensus_state)?;
        if tm_consensus_state.root().is_empty() {
            return Err(ClientError::Other {
                description: "empty commitment root".into(),
            });
        };

        Ok(())
    }

    fn client_type(&self) -> ClientType {
        sov_client_type()
    }

    fn latest_height(&self) -> Height {
        self.0.latest_height()
    }

    fn validate_proof_height(&self, proof_height: Height) -> Result<(), ClientError> {
        if self.latest_height() < proof_height {
            return Err(ClientError::InvalidProofHeight {
                latest_height: self.latest_height(),
                proof_height,
            });
        }
        Ok(())
    }

    /// Perform client-specific verifications and check all data in the new
    /// client state to be the same across all valid clients for the new chain.
    ///
    /// You can learn more about how to upgrade IBC-connected SDK chains in
    /// [this](https://ibc.cosmos.network/main/ibc/upgrades/quick-guide.html)
    /// guide
    fn verify_upgrade_client(
        &self,
        _upgraded_client_state: Any,
        _upgraded_consensus_state: Any,
        _proof_upgrade_client: CommitmentProofBytes,
        _proof_upgrade_consensus_state: CommitmentProofBytes,
        _root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn verify_membership(
        &self,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _path: Path,
        _value: Vec<u8>,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn verify_non_membership(
        &self,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _path: Path,
    ) -> Result<(), ClientError> {
        Ok(())
    }
}

impl<V> ClientStateValidation<V> for ClientState
where
    V: ClientValidationContext + SovValidationContext,
    V::AnyConsensusState: TryInto<ConsensusState>,
    ClientError: From<<V::AnyConsensusState as TryInto<ConsensusState>>::Error>,
{
    fn verify_client_message(
        &self,
        _ctx: &V,
        _client_id: &ClientId,
        _client_message: Any,
        _update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn check_for_misbehaviour(
        &self,
        _ctx: &V,
        _client_id: &ClientId,
        _client_message: Any,
        _update_kind: &UpdateKind,
    ) -> Result<bool, ClientError> {
        Ok(false)
    }

    fn status(&self, ctx: &V, client_id: &ClientId) -> Result<Status, ClientError> {
        if self.0.is_frozen() {
            return Ok(Status::Frozen);
        }

        let latest_consensus_state: ConsensusState = {
            let any_latest_consensus_state =
                match ctx.consensus_state(&ClientConsensusStatePath::new(
                    client_id.clone(),
                    self.0.latest_height.revision_number(),
                    self.0.latest_height.revision_height(),
                )) {
                    Ok(cs) => cs,
                    // if the client state does not have an associated consensus state for its latest height
                    // then it must be expired
                    Err(_) => return Ok(Status::Expired),
                };

            any_latest_consensus_state.try_into()?
        };

        // Note: if the `duration_since()` is `None`, indicating that the latest
        // consensus state is in the future, then we don't consider the client
        // to be expired.
        let now = ctx.host_timestamp()?;
        if let Some(elapsed_since_latest_consensus_state) =
            now.duration_since(&latest_consensus_state.timestamp().into())
        {
            if elapsed_since_latest_consensus_state > self.0.tendermint_params.trusting_period {
                return Ok(Status::Expired);
            }
        }

        Ok(Status::Active)
    }
}

impl<E> ClientStateExecution<E> for ClientState
where
    E: ClientExecutionContext + SovValidationContext + ExecutionContext,
    <E as ClientExecutionContext>::AnyClientState: From<ClientState>,
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

        let sov_consensus_state = ConsensusState::try_from(consensus_state)?;

        ctx.store_client_state(ClientStatePath::new(client_id), self.clone().into())?;
        ctx.store_consensus_state(
            ClientConsensusStatePath::new(
                client_id.clone(),
                self.latest_height().revision_number(),
                self.latest_height().revision_height(),
            ),
            sov_consensus_state.into(),
        )?;
        ctx.store_update_meta(
            client_id.clone(),
            self.latest_height(),
            host_timestamp,
            host_height,
        )?;
        Ok(())
    }

    fn update_state(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        header: Any,
    ) -> Result<Vec<Height>, ClientError> {
        let header = Header::try_from(header)?;
        let header_height = header.height();

        // self.prune_oldest_consensus_state(ctx, client_id)?;

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

            let new_consensus_state = ConsensusStateType::from(header.clone());
            let new_client_state = self.0.clone().with_header(header.da_header)?;

            ctx.store_consensus_state(
                ClientConsensusStatePath::new(
                    client_id.clone(),
                    new_client_state.latest_height.revision_number(),
                    new_client_state.latest_height.revision_height(),
                ),
                ConsensusState::from(new_consensus_state).into(),
            )?;
            ctx.store_client_state(
                ClientStatePath::new(client_id),
                ClientState::from(new_client_state).into(),
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

    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        _client_message: Any,
        _update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        let frozen_client_state = self.0.clone().with_frozen_height(Height::min(0));

        let wrapped_frozen_client_state = ClientState::from(frozen_client_state);

        ctx.store_client_state(
            ClientStatePath::new(client_id),
            wrapped_frozen_client_state.into(),
        )?;

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
        let mut upgraded_client_state = Self::try_from(upgraded_client_state)?;
        let upgraded_tm_cons_state = ConsensusState::try_from(upgraded_consensus_state)?;

        upgraded_client_state.0.zero_custom_fields();

        // Construct new client state and consensus state relayer chosen client
        // parameters are ignored. All chain-chosen parameters come from
        // committed client, all client-chosen parameters come from current
        // client.
        let new_client_state = ClientStateType::new(
            upgraded_client_state.0.rollup_id,
            upgraded_client_state.0.latest_height,
            upgraded_client_state.0.upgrade_path,
            upgraded_client_state.0.tendermint_params,
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

        let latest_height = new_client_state.latest_height;
        let host_timestamp = CommonContext::host_timestamp(ctx)?;
        let host_height = CommonContext::host_height(ctx)?;

        ctx.store_client_state(
            ClientStatePath::new(client_id),
            ClientState::from(new_client_state).into(),
        )?;
        ctx.store_consensus_state(
            ClientConsensusStatePath::new(
                client_id.clone(),
                latest_height.revision_number(),
                latest_height.revision_height(),
            ),
            ConsensusState::from(new_consensus_state).into(),
        )?;
        ctx.store_update_meta(
            client_id.clone(),
            latest_height,
            host_timestamp,
            host_height,
        )?;

        Ok(latest_height)
    }
}
