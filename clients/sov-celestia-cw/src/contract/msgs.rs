use alloc::vec::Vec;
use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use ibc::core::ics23_commitment::commitment::{CommitmentPrefix, CommitmentProofBytes};
use ibc::core::ics24_host::path::Path;
use ibc::proto::Any;
use ibc::Height;
use ibc_proto::ibc::core::client::v1::Height as RawHeight;
use ibc_proto::ibc::lightclients::wasm::v1::ClientMessage as RawClientMessage;
use ics08_wasm::client_state::ClientState as WasmClientState;
use ics08_wasm::consensus_state::ConsensusState as WasmConsensusState;
use prost::Message;

use super::error::ContractError;
use crate::types::client_message::{
    ClientMessage, SovHeader, SovMisbehaviour, SOVEREIGN_HEADER_TYPE_URL,
    SOVEREIGN_MISBEHAVIOUR_TYPE_URL,
};
use crate::types::serializer::Base64;
use crate::types::Bytes;

#[cw_serde]
pub struct GenesisMetadata {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    VerifyMembership(VerifyMembershipMsgRaw),
    VerifyNonMembership(VerifyNonMembershipMsgRaw),
    VerifyClientMessage(VerifyClientMessageMsgRaw),
    CheckForMisbehaviour(CheckForMisbehaviourMsgRaw),
    UpdateStateOnMisbehaviour(UpdateStateOnMisbehaviourMsgRaw),
    UpdateState(UpdateStateMsgRaw),
    CheckSubstituteAndUpdateState(CheckSubstituteAndUpdateStateMsg),
    VerifyUpgradeAndUpdateState(VerifyUpgradeAndUpdateStateMsgRaw),
}

#[cw_serde]
pub enum QueryMsg {
    ClientTypeMsg(ClientTypeMsg),
    GetLatestHeightsMsg(GetLatestHeightsMsg),
    ExportMetadata(ExportMetadataMsg),
    Status(StatusMsg),
}

#[cw_serde]
pub struct ClientTypeMsg {}

#[cw_serde]
pub struct GetLatestHeightsMsg {}

#[cw_serde]
pub struct StatusMsg {}

#[cw_serde]
pub struct ExportMetadataMsg {}

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
        let height = Height::try_from(raw.height).unwrap();
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
        let height = raw.height.try_into().unwrap();
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
pub struct WasmMisbehaviour {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub data: Bytes,
}

#[cw_serde]
pub struct VerifyClientMessageMsgRaw {
    pub client_message: RawClientMessage,
}

pub struct VerifyClientMessageMsg {
    pub client_message: ClientMessage,
}

impl TryFrom<VerifyClientMessageMsgRaw> for VerifyClientMessageMsg {
    type Error = ContractError;

    fn try_from(raw: VerifyClientMessageMsgRaw) -> Result<Self, Self::Error> {
        let client_message = Self::decode_client_message(raw.client_message)?;
        Ok(Self { client_message })
    }
}

impl VerifyClientMessageMsg {
    fn decode_client_message(raw: RawClientMessage) -> Result<ClientMessage, ContractError> {
        let maybe_any_header = Any {
            type_url: SOVEREIGN_HEADER_TYPE_URL.to_string(),
            value: raw.data.clone(),
        };

        if let Ok(header) = SovHeader::try_from(maybe_any_header) {
            return Ok(ClientMessage::Header(header));
        }

        let maybe_any_misbehaviour = Any {
            type_url: SOVEREIGN_MISBEHAVIOUR_TYPE_URL.to_string(),
            value: raw.data,
        };

        if let Ok(misbehaviour) = SovMisbehaviour::try_from(maybe_any_misbehaviour) {
            return Ok(ClientMessage::Misbehaviour(misbehaviour));
        }

        Err(ContractError::Celestia(
            "Unknown client message type".to_string(),
        ))
    }
}

#[cw_serde]
pub struct CheckForMisbehaviourMsgRaw {
    pub client_message: RawClientMessage,
}

pub struct CheckForMisbehaviourMsg {
    pub client_message: ClientMessage,
}

impl TryFrom<CheckForMisbehaviourMsgRaw> for CheckForMisbehaviourMsg {
    type Error = ContractError;

    fn try_from(raw: CheckForMisbehaviourMsgRaw) -> Result<Self, Self::Error> {
        let client_message = VerifyClientMessageMsg::decode_client_message(raw.client_message)?;
        Ok(Self { client_message })
    }
}

#[cw_serde]
pub struct UpdateStateOnMisbehaviourMsgRaw {
    pub client_message: RawClientMessage,
}

pub struct UpdateStateOnMisbehaviourMsg {
    pub client_message: ClientMessage,
}

impl TryFrom<UpdateStateOnMisbehaviourMsgRaw> for UpdateStateOnMisbehaviourMsg {
    type Error = ContractError;

    fn try_from(raw: UpdateStateOnMisbehaviourMsgRaw) -> Result<Self, Self::Error> {
        let client_message = VerifyClientMessageMsg::decode_client_message(raw.client_message)?;
        Ok(Self { client_message })
    }
}

#[cw_serde]
pub struct UpdateStateMsgRaw {
    pub client_message: RawClientMessage,
}

pub struct UpdateStateMsg {
    pub client_message: ClientMessage,
}

impl TryFrom<UpdateStateMsgRaw> for UpdateStateMsg {
    type Error = ContractError;

    fn try_from(raw: UpdateStateMsgRaw) -> Result<Self, Self::Error> {
        let client_message = VerifyClientMessageMsg::decode_client_message(raw.client_message)?;
        Ok(Self { client_message })
    }
}

#[cw_serde]
pub struct CheckSubstituteAndUpdateStateMsg {}

#[cw_serde]
pub struct VerifyUpgradeAndUpdateStateMsgRaw {
    pub upgrade_client_state: WasmClientState,
    pub upgrade_consensus_state: WasmConsensusState,
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
        let upgrade_client_state = Any::decode(&mut raw.upgrade_client_state.data.as_slice())?;

        let upgrade_consensus_state =
            Any::decode(&mut raw.upgrade_consensus_state.data.as_slice())?;

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
