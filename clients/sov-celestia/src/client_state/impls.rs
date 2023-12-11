//! Implements the core [`ClientState`](crate::core::ics02_client::client_state::ClientState) trait
//! for the Sovereign light client.

use alloc::vec;
use alloc::vec::Vec;
use core::convert::{TryFrom, TryInto};
use core::str::FromStr;

use ibc_core::client::context::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation,
};
use ibc_core::client::context::consensus_state::ConsensusState;
use ibc_core::client::context::{ClientExecutionContext, ClientValidationContext};
use ibc_core::client::types::error::{ClientError, UpgradeClientError};
use ibc_core::client::types::{Height, Status, UpdateKind};
use ibc_core::commitment_types::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use ibc_core::commitment_types::merkle::{apply_prefix, MerkleProof};
use ibc_core::host::types::identifiers::{ChainId, ClientId, ClientType};
use ibc_core::host::types::path::{
    ClientConsensusStatePath, ClientStatePath, Path, UpgradeClientPath,
};
use ibc_core::primitives::ToVec;
use ibc_proto::google::protobuf::Any;
use tendermint_proto::Protobuf;

use super::definition::{AllowUpdate, ClientState};
use super::sov_client_type;
use crate::client_message::{SovHeader, SovMisbehaviour};
use crate::consensus_state::SovConsensusState;
use crate::context::ValidationContext as SovValidationContext;
use crate::error::Error;
use crate::proto::ClientState as RawSovClientState;

impl ClientStateCommon for ClientState {
    fn verify_consensus_state(&self, consensus_state: Any) -> Result<(), ClientError> {
        let tm_consensus_state =
            SovConsensusState::try_from(consensus_state).map_err(|e| ClientError::Other {
                description: e.to_string(),
            })?;
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
        self.latest_height
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
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
        proof_upgrade_client: CommitmentProofBytes,
        proof_upgrade_consensus_state: CommitmentProofBytes,
        root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        // Make sure that the client type is of Sovereign type `ClientState`
        let upgraded_sov_client_state = Self::try_from(upgraded_client_state.clone())?;

        // Make sure that the consensus type is of Sovereign type `ConsensusState`
        SovConsensusState::try_from(upgraded_consensus_state.clone()).map_err(|e| {
            ClientError::Other {
                description: e.to_string(),
            }
        })?;

        // Make sure the latest height of the current client is not greater then
        // the upgrade height This condition checks both the revision number and
        // the height
        if self.latest_height() >= upgraded_sov_client_state.latest_height {
            return Err(UpgradeClientError::LowUpgradeHeight {
                upgraded_height: self.latest_height(),
                client_height: upgraded_sov_client_state.latest_height,
            })?;
        }

        // Check to see if the upgrade path is set
        let mut upgrade_path = self.upgrade_path.clone();
        if upgrade_path.pop().is_none() {
            return Err(ClientError::ClientSpecific {
                description: "cannot upgrade client as no upgrade path has been set".to_string(),
            });
        };

        let upgrade_path_prefix = CommitmentPrefix::try_from(upgrade_path[0].clone().into_bytes())
            .map_err(ClientError::InvalidCommitmentProof)?;

        let last_height = self.latest_height().revision_height();

        // Verify the proof of the upgraded client state
        self.verify_membership(
            &upgrade_path_prefix,
            &proof_upgrade_client,
            root,
            Path::UpgradeClient(UpgradeClientPath::UpgradedClientState(last_height)),
            upgraded_client_state.to_vec(),
        )?;

        // Verify the proof of the upgraded consensus state
        self.verify_membership(
            &upgrade_path_prefix,
            &proof_upgrade_consensus_state,
            root,
            Path::UpgradeClient(UpgradeClientPath::UpgradedClientConsensusState(last_height)),
            upgraded_consensus_state.to_vec(),
        )?;

        Ok(())
    }

    fn verify_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
        value: Vec<u8>,
    ) -> Result<(), ClientError> {
        let merkle_path = apply_prefix(prefix, vec![path.to_string()]);
        let merkle_proof =
            MerkleProof::try_from(proof.clone()).map_err(ClientError::InvalidCommitmentProof)?;

        merkle_proof
            .verify_membership(
                &self.proof_specs,
                root.clone().into(),
                merkle_path,
                value,
                0,
            )
            .map_err(ClientError::Ics23Verification)
    }

    fn verify_non_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
    ) -> Result<(), ClientError> {
        let merkle_path = apply_prefix(prefix, vec![path.to_string()]);
        let merkle_proof =
            MerkleProof::try_from(proof.clone()).map_err(ClientError::InvalidCommitmentProof)?;

        merkle_proof
            .verify_non_membership(&self.proof_specs, root.clone().into(), merkle_path)
            .map_err(ClientError::Ics23Verification)
    }
}

impl<V> ClientStateValidation<V> for ClientState
where
    V: ClientValidationContext + SovValidationContext,
    V::AnyConsensusState: TryInto<SovConsensusState>,
    ClientError: From<<V::AnyConsensusState as TryInto<SovConsensusState>>::Error>,
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
                self.verify_header(ctx, client_id, header)
            }
            UpdateKind::SubmitMisbehaviour => {
                let misbehaviour = SovMisbehaviour::try_from(client_message)?;
                self.verify_misbehaviour(ctx, client_id, misbehaviour)
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
                self.check_for_misbehaviour_update_client(ctx, client_id, header)
            }
            UpdateKind::SubmitMisbehaviour => {
                let misbehaviour = SovMisbehaviour::try_from(client_message)?;
                self.check_for_misbehaviour_misbehavior(&misbehaviour)
            }
        }
    }

    fn status(&self, ctx: &V, client_id: &ClientId) -> Result<Status, ClientError> {
        if self.is_frozen() {
            return Ok(Status::Frozen);
        }

        let latest_consensus_state: SovConsensusState = {
            let any_latest_consensus_state =
                match ctx.consensus_state(&ClientConsensusStatePath::new(
                    client_id.clone(),
                    self.latest_height.revision_number(),
                    self.latest_height.revision_height(),
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
            now.duration_since(&latest_consensus_state.timestamp())
        {
            if elapsed_since_latest_consensus_state > self.trusting_period {
                return Ok(Status::Expired);
            }
        }

        Ok(Status::Active)
    }
}

impl<E> ClientStateExecution<E> for ClientState
where
    E: ClientExecutionContext + SovValidationContext,
    <E as ClientExecutionContext>::AnyClientState: From<ClientState>,
    <E as ClientExecutionContext>::AnyConsensusState: From<SovConsensusState>,
{
    fn initialise(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        consensus_state: Any,
    ) -> Result<(), ClientError> {
        let tm_consensus_state = SovConsensusState::try_from(consensus_state)?;

        ctx.store_client_state(ClientStatePath::new(client_id), self.clone().into())?;
        ctx.store_consensus_state(
            ClientConsensusStatePath::new(
                client_id.clone(),
                self.latest_height.revision_number(),
                self.latest_height.revision_height(),
            ),
            tm_consensus_state.into(),
        )?;
        ctx.store_update_time(
            client_id.clone(),
            self.latest_height(),
            ctx.host_timestamp()?,
        )?;
        ctx.store_update_height(client_id.clone(), self.latest_height(), ctx.host_height()?)?;

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
            let new_consensus_state = SovConsensusState::from(header.clone());
            let new_client_state = self.clone().with_header(header)?;

            ctx.store_consensus_state(
                ClientConsensusStatePath::new(
                    client_id.clone(),
                    new_client_state.latest_height.revision_number(),
                    new_client_state.latest_height.revision_height(),
                ),
                new_consensus_state.into(),
            )?;
            ctx.store_client_state(ClientStatePath::new(client_id), new_client_state.into())?;
            ctx.store_update_time(client_id.clone(), header_height, ctx.host_timestamp()?)?;
            ctx.store_update_height(client_id.clone(), header_height, ctx.host_height()?)?;
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
        let frozen_client_state = self.clone().with_frozen_height(Height::min(0));

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
        let mut upgraded_tm_client_state = Self::try_from(upgraded_client_state)?;
        let upgraded_tm_cons_state = SovConsensusState::try_from(upgraded_consensus_state)?;

        upgraded_tm_client_state.zero_custom_fields();

        // Construct new client state and consensus state relayer chosen client
        // parameters are ignored. All chain-chosen parameters come from
        // committed client, all client-chosen parameters come from current
        // client.
        let new_client_state = ClientState::new(
            upgraded_tm_client_state.chain_id,
            self.trust_level,
            self.trusting_period,
            upgraded_tm_client_state.unbonding_period,
            self.max_clock_drift,
            upgraded_tm_client_state.latest_height,
            upgraded_tm_client_state.proof_specs,
            upgraded_tm_client_state.upgrade_path,
            self.allow_update,
        )
        .map_err(|e| ClientError::Other {
            description: e.to_string(),
        })?;

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
        let new_consensus_state = SovConsensusState::new(
            sentinel_root.into(),
            upgraded_tm_cons_state.timestamp,
            upgraded_tm_cons_state.next_validators_hash,
        );

        let latest_height = new_client_state.latest_height;

        ctx.store_client_state(ClientStatePath::new(client_id), new_client_state.into())?;
        ctx.store_consensus_state(
            ClientConsensusStatePath::new(
                client_id.clone(),
                latest_height.revision_number(),
                latest_height.revision_height(),
            ),
            new_consensus_state.into(),
        )?;
        ctx.store_update_time(client_id.clone(), latest_height, ctx.host_timestamp()?)?;
        ctx.store_update_height(client_id.clone(), latest_height, ctx.host_height()?)?;

        Ok(latest_height)
    }
}

impl Protobuf<RawSovClientState> for ClientState {}

impl TryFrom<RawSovClientState> for ClientState {
    type Error = Error;

    fn try_from(raw: RawSovClientState) -> Result<Self, Self::Error> {
        let chain_id = ChainId::from_str(raw.chain_id.as_str())?;

        let trust_level = {
            let trust_level = raw
                .trust_level
                .clone()
                .ok_or(Error::missing("trust_level"))?;
            trust_level
                .try_into()
                .map_err(|_| Error::invalid("trust_level"))?
        };

        let trusting_period = raw
            .trusting_period
            .ok_or(Error::missing("trusting_period"))?
            .try_into()
            .map_err(|_| Error::invalid("trusting_period"))?;

        let unbonding_period = raw
            .unbonding_period
            .ok_or(Error::missing("unbonding_period"))?
            .try_into()
            .map_err(|_| Error::invalid("unbonding_period"))?;

        let max_clock_drift = raw
            .max_clock_drift
            .ok_or(Error::invalid("negative max clock drift"))?
            .try_into()
            .map_err(|_| Error::invalid("negative max clock drift"))?;

        let latest_height = raw
            .latest_height
            .ok_or(Error::missing("latest_height"))?
            .try_into()
            .map_err(|_| Error::invalid("latest_height"))?;

        // In `RawClientState`, a `frozen_height` of `0` means "not frozen".
        // See:
        // https://github.com/cosmos/ibc-go/blob/8422d0c4c35ef970539466c5bdec1cd27369bab3/modules/light-clients/07-tendermint/types/client_state.go#L74
        if raw
            .frozen_height
            .and_then(|h| Height::try_from(h).ok())
            .is_some()
        {
            return Err(Error::not_allowed("frozen_height"));
        }

        // We use set this deprecated field just so that we can properly convert
        // it back in its raw form
        #[allow(deprecated)]
        let allow_update = AllowUpdate {
            after_expiry: raw.allow_update_after_expiry,
            after_misbehaviour: raw.allow_update_after_misbehaviour,
        };

        let client_state = Self::new_without_validation(
            chain_id,
            trust_level,
            trusting_period,
            unbonding_period,
            max_clock_drift,
            latest_height,
            raw.proof_specs.into(),
            raw.upgrade_path,
            allow_update,
        );

        Ok(client_state)
    }
}
