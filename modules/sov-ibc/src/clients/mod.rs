pub mod context;

use derive_more::TryInto;
use ibc_client_tendermint::client_state::ClientState as TmClientState;
use ibc_client_tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc_client_tendermint::types::{
    ClientState as TmClientStateTypes, ConsensusState as TmConsensusStateType,
    TENDERMINT_CLIENT_STATE_TYPE_URL, TENDERMINT_CONSENSUS_STATE_TYPE_URL,
};
use ibc_core::client::context::prelude::*;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::{Height, Status};
use ibc_core::commitment_types::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use ibc_core::derive::ConsensusState;
use ibc_core::host::types::identifiers::{ClientId, ClientType};
use ibc_core::host::types::path::Path;
use ibc_core::primitives::proto::{Any, Protobuf};
use sov_celestia_client::client_state::ClientState as SovClientState;
use sov_celestia_client::consensus_state::ConsensusState as SovConsensusState;
use sov_celestia_client::types::client_state::{
    SovTmClientState, SOV_TENDERMINT_CLIENT_STATE_TYPE_URL,
};
use sov_celestia_client::types::consensus_state::{
    SovTmConsensusState, SOV_TENDERMINT_CONSENSUS_STATE_TYPE_URL,
};
use sov_modules_api::{Spec, TxState};

use crate::context::IbcContext;

#[derive(Clone, Debug, TryInto)]
pub enum AnyClientState {
    Tendermint(TmClientState),
    Sovereign(SovClientState),
}

impl From<SovTmClientState> for AnyClientState {
    fn from(cs: SovTmClientState) -> Self {
        Self::Sovereign(cs.into())
    }
}

impl From<TmClientStateTypes> for AnyClientState {
    fn from(cs: TmClientStateTypes) -> Self {
        Self::Tendermint(cs.into())
    }
}

impl Protobuf<Any> for AnyClientState {}

impl TryFrom<Any> for AnyClientState {
    type Error = ClientError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            TENDERMINT_CLIENT_STATE_TYPE_URL => {
                let tm_cs: TmClientState = value.try_into()?;
                Ok(Self::Tendermint(tm_cs))
            }
            SOV_TENDERMINT_CLIENT_STATE_TYPE_URL => {
                let sov_cs: SovClientState = value.try_into()?;
                Ok(Self::Sovereign(sov_cs))
            }
            _ => Err(ClientError::UnknownClientStateType {
                client_state_type: value.type_url,
            }),
        }
    }
}

impl From<AnyClientState> for Any {
    fn from(any_cs: AnyClientState) -> Self {
        match any_cs {
            AnyClientState::Tendermint(tm_cs) => tm_cs.into(),
            AnyClientState::Sovereign(sov_cs) => sov_cs.into(),
        }
    }
}

// Next 3 trait impls are boilerplate
// We have a `ClientState` macro, but unfortunately it doesn't currently support
// the context (`IbcExecutionContext` in this case) to be generic
impl ClientStateCommon for AnyClientState {
    fn verify_consensus_state(&self, consensus_state: Any) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.verify_consensus_state(consensus_state),
            AnyClientState::Sovereign(cs) => cs.verify_consensus_state(consensus_state),
        }
    }

    fn client_type(&self) -> ClientType {
        match self {
            AnyClientState::Tendermint(cs) => cs.client_type(),
            AnyClientState::Sovereign(cs) => cs.client_type(),
        }
    }

    fn latest_height(&self) -> Height {
        match self {
            AnyClientState::Tendermint(cs) => cs.latest_height(),
            AnyClientState::Sovereign(cs) => cs.latest_height(),
        }
    }

    fn validate_proof_height(&self, proof_height: Height) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.validate_proof_height(proof_height),
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
            AnyClientState::Tendermint(cs) => cs.verify_upgrade_client(
                upgraded_client_state,
                upgraded_consensus_state,
                proof_upgrade_client,
                proof_upgrade_consensus_state,
                root,
            ),
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
            AnyClientState::Tendermint(cs) => {
                cs.verify_membership(prefix, proof, root, path, value)
            }
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
            AnyClientState::Tendermint(cs) => cs.verify_non_membership(prefix, proof, root, path),
            AnyClientState::Sovereign(cs) => cs.verify_non_membership(prefix, proof, root, path),
        }
    }
}

impl<'a, S, TS> ClientStateExecution<IbcContext<'a, S, TS>> for AnyClientState
where
    S: Spec,
    TS: TxState<S>,
{
    fn initialise(
        &self,
        ctx: &mut IbcContext<'a, S, TS>,
        client_id: &ClientId,
        consensus_state: Any,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.initialise(ctx, client_id, consensus_state),
            AnyClientState::Sovereign(cs) => cs.initialise(ctx, client_id, consensus_state),
        }
    }

    fn update_state(
        &self,
        ctx: &mut IbcContext<'a, S, TS>,
        client_id: &ClientId,
        header: Any,
    ) -> Result<Vec<Height>, ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.update_state(ctx, client_id, header),
            AnyClientState::Sovereign(cs) => cs.update_state(ctx, client_id, header),
        }
    }

    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut IbcContext<'a, S, TS>,
        client_id: &ClientId,
        client_message: Any,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => {
                cs.update_state_on_misbehaviour(ctx, client_id, client_message)
            }
            AnyClientState::Sovereign(cs) => {
                cs.update_state_on_misbehaviour(ctx, client_id, client_message)
            }
        }
    }

    fn update_state_on_upgrade(
        &self,
        ctx: &mut IbcContext<'a, S, TS>,
        client_id: &ClientId,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
    ) -> Result<Height, ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.update_state_on_upgrade(
                ctx,
                client_id,
                upgraded_client_state,
                upgraded_consensus_state,
            ),
            AnyClientState::Sovereign(cs) => cs.update_state_on_upgrade(
                ctx,
                client_id,
                upgraded_client_state,
                upgraded_consensus_state,
            ),
        }
    }

    fn update_on_recovery(
        &self,
        ctx: &mut IbcContext<'a, S, TS>,
        subject_client_id: &ClientId,
        substitute_client_state: Any,
        substitute_consensus_state: Any,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.update_on_recovery(
                ctx,
                subject_client_id,
                substitute_client_state,
                substitute_consensus_state,
            ),
            AnyClientState::Sovereign(cs) => cs.update_on_recovery(
                ctx,
                subject_client_id,
                substitute_client_state,
                substitute_consensus_state,
            ),
        }
    }
}

impl<'a, S, TS> ClientStateValidation<IbcContext<'a, S, TS>> for AnyClientState
where
    S: Spec,
    TS: TxState<S>,
{
    fn verify_client_message(
        &self,
        ctx: &IbcContext<'a, S, TS>,
        client_id: &ClientId,
        client_message: Any,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => {
                cs.verify_client_message(ctx, client_id, client_message)
            }
            AnyClientState::Sovereign(cs) => {
                cs.verify_client_message(ctx, client_id, client_message)
            }
        }
    }

    fn check_for_misbehaviour(
        &self,
        ctx: &IbcContext<'a, S, TS>,
        client_id: &ClientId,
        client_message: Any,
    ) -> Result<bool, ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => {
                cs.check_for_misbehaviour(ctx, client_id, client_message)
            }
            AnyClientState::Sovereign(cs) => {
                cs.check_for_misbehaviour(ctx, client_id, client_message)
            }
        }
    }

    fn status(
        &self,
        ctx: &IbcContext<'a, S, TS>,
        client_id: &ClientId,
    ) -> Result<Status, ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.status(ctx, client_id),
            AnyClientState::Sovereign(cs) => cs.status(ctx, client_id),
        }
    }

    fn check_substitute(
        &self,
        ctx: &IbcContext<'a, S, TS>,
        substitute_client_state: Any,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(cs) => cs.check_substitute(ctx, substitute_client_state),
            AnyClientState::Sovereign(cs) => cs.check_substitute(ctx, substitute_client_state),
        }
    }
}

#[derive(Clone, ConsensusState)]
pub enum AnyConsensusState {
    Tendermint(TmConsensusState),
    Sovereign(SovConsensusState),
}

impl TryFrom<AnyConsensusState> for TmConsensusStateType {
    type Error = ClientError;

    fn try_from(cs: AnyConsensusState) -> Result<TmConsensusStateType, Self::Error> {
        match cs {
            AnyConsensusState::Tendermint(cs) => Ok(cs.into_inner()),
            _ => Err(ClientError::UnknownConsensusStateType {
                consensus_state_type: "".to_string(),
            }),
        }
    }
}

impl TryFrom<AnyConsensusState> for SovTmConsensusState {
    type Error = ClientError;

    fn try_from(cs: AnyConsensusState) -> Result<SovTmConsensusState, Self::Error> {
        match cs {
            AnyConsensusState::Sovereign(cs) => Ok(cs.into_inner()),
            _ => Err(ClientError::UnknownConsensusStateType {
                consensus_state_type: "".to_string(),
            }),
        }
    }
}

impl From<SovTmConsensusState> for AnyConsensusState {
    fn from(cs: SovTmConsensusState) -> Self {
        Self::Sovereign(cs.into())
    }
}

impl From<TmConsensusStateType> for AnyConsensusState {
    fn from(cs: TmConsensusStateType) -> Self {
        Self::Tendermint(cs.into())
    }
}

impl Protobuf<Any> for AnyConsensusState {}

impl TryFrom<Any> for AnyConsensusState {
    type Error = ClientError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            TENDERMINT_CONSENSUS_STATE_TYPE_URL => {
                let tm_cs: TmConsensusState = value.try_into()?;
                Ok(Self::Tendermint(tm_cs))
            }
            SOV_TENDERMINT_CONSENSUS_STATE_TYPE_URL => {
                let sov_cs: SovConsensusState = value.try_into()?;
                Ok(Self::Sovereign(sov_cs))
            }
            _ => Err(ClientError::UnknownConsensusStateType {
                consensus_state_type: value.type_url,
            }),
        }
    }
}

impl From<AnyConsensusState> for Any {
    fn from(any_cs: AnyConsensusState) -> Self {
        match any_cs {
            AnyConsensusState::Tendermint(tm_cs) => tm_cs.into(),
            AnyConsensusState::Sovereign(sov_cs) => sov_cs.into(),
        }
    }
}
