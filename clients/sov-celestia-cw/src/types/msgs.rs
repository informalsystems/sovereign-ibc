//! Contains the definition of the messages that can be sent to the CosmWasm contract.
use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use ibc_client_wasm_types::serializer::Base64;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::proto::v1::Height as RawHeight;
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::{CommitmentPrefix, CommitmentProofBytes};
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::path::Path;
use ibc_core::primitives::proto::Any;
use prost::Message;
use sov_celestia_client::types::client_message::SovTmClientMessage;

use super::error::ContractError;

pub type Bytes = Vec<u8>;

// ------------------------------------------------------------
// Implementation of the InstantiateMsg struct
// ------------------------------------------------------------

#[cw_serde]
pub struct InstantiateMsg {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub client_state: Bytes,
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub consensus_state: Bytes,
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub checksum: Bytes,
}

// ------------------------------------------------------------
// Implementation of the SudoMsg enum and its variants
// ------------------------------------------------------------

#[derive(derive_more::From)]
#[cw_serde]
pub enum SudoMsg {
    UpdateState(UpdateStateMsgRaw),
    UpdateStateOnMisbehaviour(UpdateStateOnMisbehaviourMsgRaw),
    VerifyUpgradeAndUpdateState(VerifyUpgradeAndUpdateStateMsgRaw),
    VerifyMembership(VerifyMembershipMsgRaw),
    VerifyNonMembership(VerifyNonMembershipMsgRaw),
    MigrateClientStore(MigrateClientStoreMsg),
}

#[cw_serde]
pub struct UpdateStateOnMisbehaviourMsgRaw {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub client_message: Bytes,
}

pub struct UpdateStateOnMisbehaviourMsg {
    pub client_message: SovTmClientMessage,
}

impl TryFrom<UpdateStateOnMisbehaviourMsgRaw> for UpdateStateOnMisbehaviourMsg {
    type Error = ContractError;

    fn try_from(raw: UpdateStateOnMisbehaviourMsgRaw) -> Result<Self, Self::Error> {
        let client_message = SovTmClientMessage::decode(raw.client_message)?;

        Ok(Self { client_message })
    }
}

#[cw_serde]
pub struct UpdateStateMsgRaw {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub client_message: Bytes,
}

pub struct UpdateStateMsg {
    pub client_message: SovTmClientMessage,
}

impl TryFrom<UpdateStateMsgRaw> for UpdateStateMsg {
    type Error = ContractError;

    fn try_from(raw: UpdateStateMsgRaw) -> Result<Self, Self::Error> {
        let client_message = SovTmClientMessage::decode(raw.client_message)?;
        Ok(Self { client_message })
    }
}

#[cw_serde]
pub struct CheckSubstituteAndUpdateStateMsg {}

#[cw_serde]
pub struct VerifyUpgradeAndUpdateStateMsgRaw {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub upgrade_client_state: Bytes,
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub upgrade_consensus_state: Bytes,
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub proof_upgrade_client: Bytes,
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub proof_upgrade_consensus_state: Bytes,
}

pub struct VerifyUpgradeAndUpdateStateMsg {
    pub upgrade_client_state: Any,
    pub upgrade_consensus_state: Any,
    pub proof_upgrade_client: CommitmentProofBytes,
    pub proof_upgrade_consensus_state: CommitmentProofBytes,
}

impl TryFrom<VerifyUpgradeAndUpdateStateMsgRaw> for VerifyUpgradeAndUpdateStateMsg {
    type Error = ContractError;

    fn try_from(raw: VerifyUpgradeAndUpdateStateMsgRaw) -> Result<Self, Self::Error> {
        let upgrade_client_state = Any::decode(&mut raw.upgrade_client_state.as_slice())?;

        let upgrade_consensus_state = Any::decode(&mut raw.upgrade_consensus_state.as_slice())?;

        Ok(VerifyUpgradeAndUpdateStateMsg {
            upgrade_client_state,
            upgrade_consensus_state,
            proof_upgrade_client: CommitmentProofBytes::try_from(raw.proof_upgrade_client)?,
            proof_upgrade_consensus_state: CommitmentProofBytes::try_from(
                raw.proof_upgrade_consensus_state,
            )?,
        })
    }
}

#[cw_serde]
pub struct MerklePath {
    pub key_path: Vec<String>,
}

#[cw_serde]
pub struct VerifyMembershipMsgRaw {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub proof: Bytes,
    pub path: MerklePath,
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub value: Bytes,
    pub height: RawHeight,
    pub delay_block_period: u64,
    pub delay_time_period: u64,
}

pub struct VerifyMembershipMsg {
    pub prefix: CommitmentPrefix,
    pub proof: CommitmentProofBytes,
    pub path: Path,
    pub value: Vec<u8>,
    pub height: Height,
    pub delay_block_period: u64,
    pub delay_time_period: u64,
}

impl TryFrom<VerifyMembershipMsgRaw> for VerifyMembershipMsg {
    type Error = ContractError;

    fn try_from(mut raw: VerifyMembershipMsgRaw) -> Result<Self, Self::Error> {
        let proof = CommitmentProofBytes::try_from(raw.proof)?;
        let prefix = raw.path.key_path.remove(0).into_bytes();
        let path_str = raw.path.key_path.join("");
        let path = Path::from_str(&path_str)?;
        let height = Height::try_from(raw.height).map_err(|e| {
            ContractError::Context(ContextError::ClientError(ClientError::Other {
                description: e.to_string(),
            }))
        })?;
        Ok(Self {
            proof,
            path,
            value: raw.value,
            height,
            prefix: CommitmentPrefix::try_from(prefix)?,
            delay_block_period: raw.delay_block_period,
            delay_time_period: raw.delay_time_period,
        })
    }
}

#[cw_serde]
pub struct VerifyNonMembershipMsgRaw {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub proof: Bytes,
    pub path: MerklePath,
    pub height: RawHeight,
    pub delay_block_period: u64,
    pub delay_time_period: u64,
}

pub struct VerifyNonMembershipMsg {
    pub prefix: CommitmentPrefix,
    pub proof: CommitmentProofBytes,
    pub path: Path,
    pub height: Height,
    pub delay_block_period: u64,
    pub delay_time_period: u64,
}

impl TryFrom<VerifyNonMembershipMsgRaw> for VerifyNonMembershipMsg {
    type Error = ContractError;

    fn try_from(mut raw: VerifyNonMembershipMsgRaw) -> Result<Self, Self::Error> {
        let proof = CommitmentProofBytes::try_from(raw.proof)?;
        let prefix = raw.path.key_path.remove(0).into_bytes();
        let path_str = raw.path.key_path.join("");
        let path = Path::from_str(&path_str)?;
        let height = raw.height.try_into().expect("invalid height");
        Ok(Self {
            proof,
            path,
            height,
            prefix: CommitmentPrefix::try_from(prefix)?,
            delay_block_period: raw.delay_block_period,
            delay_time_period: raw.delay_time_period,
        })
    }
}

#[cw_serde]
pub struct MigrateClientStoreMsg {}

// ------------------------------------------------------------
// Implementation of the QueryMsg enum and its variants
// ------------------------------------------------------------

#[derive(derive_more::From)]
#[cw_serde]
pub enum QueryMsg {
    Status(StatusMsg),
    ExportMetadata(ExportMetadataMsg),
    TimestampAtHeight(TimestampAtHeightMsg),
    VerifyClientMessage(VerifyClientMessageRaw),
    CheckForMisbehaviour(CheckForMisbehaviourMsgRaw),
}

#[cw_serde]
pub struct StatusMsg {}

#[cw_serde]
pub struct ExportMetadataMsg {}

#[cw_serde]
pub struct TimestampAtHeightMsg {
    pub height: Height,
}

#[cw_serde]
pub struct VerifyClientMessageRaw {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub client_message: Bytes,
}

pub struct VerifyClientMessageMsg {
    pub client_message: SovTmClientMessage,
}

impl TryFrom<VerifyClientMessageRaw> for VerifyClientMessageMsg {
    type Error = ContractError;

    fn try_from(raw: VerifyClientMessageRaw) -> Result<Self, Self::Error> {
        let client_message = SovTmClientMessage::decode(raw.client_message)?;

        Ok(Self { client_message })
    }
}

#[cw_serde]
pub struct CheckForMisbehaviourMsgRaw {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub client_message: Bytes,
}

pub struct CheckForMisbehaviourMsg {
    pub client_message: SovTmClientMessage,
}

impl TryFrom<CheckForMisbehaviourMsgRaw> for CheckForMisbehaviourMsg {
    type Error = ContractError;

    fn try_from(raw: CheckForMisbehaviourMsgRaw) -> Result<Self, Self::Error> {
        let client_message = SovTmClientMessage::decode(raw.client_message)?;

        Ok(Self { client_message })
    }
}
