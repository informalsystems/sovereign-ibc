mod definition;
mod impls;
mod misbehaviour;
mod update;

use alloc::str::FromStr;
use alloc::vec::Vec;

pub use definition::{AllowUpdate, ClientState as SovClientState, SOVEREIGN_CLIENT_STATE_TYPE_URL};
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
use ibc_core::host::types::path::Path;
use ibc_core::primitives::proto::Any;
use tendermint_proto::Protobuf;

use super::consensus_state::SovConsensusState;
use crate::context::ValidationContext as SovValidationContext;

pub const SOVEREIGN_CLIENT_TYPE: &str = "11-sovereign";

/// Returns the tendermint `ClientType`
pub fn sov_client_type() -> ClientType {
    ClientType::from_str(SOVEREIGN_CLIENT_TYPE).expect("Never fails because it's valid")
}

pub enum AnyClientState {
    Sovereign(SovClientState),
}

impl From<SovClientState> for AnyClientState {
    fn from(state: SovClientState) -> Self {
        Self::Sovereign(state)
    }
}

impl TryFrom<AnyClientState> for SovClientState {
    type Error = ClientError;

    fn try_from(value: AnyClientState) -> Result<Self, Self::Error> {
        match value {
            AnyClientState::Sovereign(state) => Ok(state),
        }
    }
}

impl From<AnyClientState> for Any {
    fn from(value: AnyClientState) -> Self {
        match value {
            AnyClientState::Sovereign(cs) => Any {
                type_url: SOVEREIGN_CLIENT_STATE_TYPE_URL.to_string(),
                value: Protobuf::<Any>::encode_vec(&cs).unwrap(),
            },
        }
    }
}

// Next 3 trait impls are boilerplate
// We have a `ClientState` macro, but unfortunately it doesn't currently support
// the context (`IbcExecutionContext` in this case) to be generic
impl ClientStateCommon for AnyClientState {
    fn verify_consensus_state(&self, consensus_state: Any) -> Result<(), ClientError> {
        match self {
            AnyClientState::Sovereign(cs) => cs.verify_consensus_state(consensus_state),
        }
    }

    fn client_type(&self) -> ClientType {
        match self {
            AnyClientState::Sovereign(cs) => cs.client_type(),
        }
    }

    fn latest_height(&self) -> Height {
        match self {
            AnyClientState::Sovereign(cs) => cs.latest_height(),
        }
    }

    fn validate_proof_height(&self, proof_height: Height) -> Result<(), ClientError> {
        match self {
            AnyClientState::Sovereign(cs) => cs.validate_proof_height(proof_height),
        }
    }

    fn verify_upgrade_client(
        &self,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
        proof_upgrade_client: CommitmentProofBytes,
        proof_upgrade_consensus_state: CommitmentProofBytes,
        root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Sovereign(cs) => cs.verify_upgrade_client(
                upgraded_client_state,
                upgraded_consensus_state,
                proof_upgrade_client,
                proof_upgrade_consensus_state,
                root,
            ),
        }
    }

    fn verify_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
        value: Vec<u8>,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Sovereign(cs) => cs.verify_membership(prefix, proof, root, path, value),
        }
    }

    fn verify_non_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Sovereign(cs) => cs.verify_non_membership(prefix, proof, root, path),
        }
    }
}

impl<Ctx> ClientStateExecution<Ctx> for AnyClientState
where
    Ctx: ClientExecutionContext + SovValidationContext,
    <Ctx as ClientExecutionContext>::AnyClientState: From<SovClientState>,
    <Ctx as ClientExecutionContext>::AnyConsensusState: From<SovConsensusState>,
{
    fn initialise(
        &self,
        ctx: &mut Ctx,
        client_id: &ClientId,
        consensus_state: Any,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Sovereign(cs) => cs.initialise(ctx, client_id, consensus_state),
        }
    }

    fn update_state(
        &self,
        ctx: &mut Ctx,
        client_id: &ClientId,
        header: Any,
    ) -> Result<Vec<Height>, ClientError> {
        match self {
            AnyClientState::Sovereign(cs) => cs.update_state(ctx, client_id, header),
        }
    }

    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut Ctx,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Sovereign(cs) => {
                cs.update_state_on_misbehaviour(ctx, client_id, client_message, update_kind)
            }
        }
    }

    fn update_state_on_upgrade(
        &self,
        ctx: &mut Ctx,
        client_id: &ClientId,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
    ) -> Result<Height, ClientError> {
        match self {
            AnyClientState::Sovereign(cs) => cs.update_state_on_upgrade(
                ctx,
                client_id,
                upgraded_client_state,
                upgraded_consensus_state,
            ),
        }
    }
}

impl<Ctx> ClientStateValidation<Ctx> for AnyClientState
where
    Ctx: ClientValidationContext + SovValidationContext,
    Ctx::AnyConsensusState: TryInto<SovConsensusState>,
    ClientError: From<<Ctx::AnyConsensusState as TryInto<SovConsensusState>>::Error>,
{
    fn verify_client_message(
        &self,
        ctx: &Ctx,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Sovereign(cs) => {
                cs.verify_client_message(ctx, client_id, client_message, update_kind)
            }
        }
    }

    fn check_for_misbehaviour(
        &self,
        ctx: &Ctx,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<bool, ClientError> {
        match self {
            AnyClientState::Sovereign(cs) => {
                cs.check_for_misbehaviour(ctx, client_id, client_message, update_kind)
            }
        }
    }

    fn status(&self, ctx: &Ctx, client_id: &ClientId) -> Result<Status, ClientError> {
        match self {
            AnyClientState::Sovereign(cs) => cs.status(ctx, client_id),
        }
    }
}
