use std::str::FromStr;

use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, Storage};
use ibc_core::client::context::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation,
};
use ibc_core::client::context::consensus_state::ConsensusState as _;
use ibc_core::client::types::UpdateKind;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::error::IdentifierError;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::ClientConsensusStatePath;
use ibc_core::host::ValidationContext;
use ibc_proto::google::protobuf::Any;
use prost::Message;
use sov_celestia_client::client_state::ClientState;
use sov_celestia_client::types::client_message::ClientMessage;

use crate::types::{
    CheckForMisbehaviourMsg, ContractError, ContractResult, ExecuteMsg, ExportMetadataMsg,
    InstantiateMsg, QueryMsg, QueryResponse, ReadonlyProcessedStates, StatusMsg, UpdateStateMsg,
    UpdateStateOnMisbehaviourMsg, VerifyClientMessageMsg, VerifyMembershipMsg,
    VerifyNonMembershipMsg, VerifyUpgradeAndUpdateStateMsg,
};

/// Context is a wrapper around the deps and env that gives access to the
/// methods of the ibc-rs Validation and Execution traits.
pub struct ContextRef<'a> {
    deps: Deps<'a>,
    env: Env,
}

impl<'a> ContextRef<'a> {
    pub fn new(deps: Deps<'a>, env: Env) -> Self {
        Self { deps, env }
    }

    pub fn env(&self) -> &Env {
        &self.env
    }

    pub fn log(&self, msg: &str) {
        self.deps.api.debug(msg)
    }

    pub fn client_id(&self) -> Result<ClientId, IdentifierError> {
        ClientId::from_str(self.env.contract.address.as_str())
    }

    pub fn query(&self, msg: QueryMsg) -> Result<Binary, ContractError> {
        let client_id = self.client_id()?;

        let resp = match msg {
            QueryMsg::ClientTypeMsg(_) => unimplemented!("ClientTypeMsg"),
            QueryMsg::GetLatestHeightsMsg(_) => unimplemented!("GetLatestHeightsMsg"),
            QueryMsg::ExportMetadata(ExportMetadataMsg {}) => {
                let ro_proceeded_state = ReadonlyProcessedStates::new(self.deps.storage);
                QueryResponse::genesis_metadata(ro_proceeded_state.get_metadata())
            }
            QueryMsg::Status(StatusMsg {}) => {
                let client_state = self.client_state(&client_id)?;

                match client_state.status(self, &client_id) {
                    Ok(status) => QueryResponse::status(status.to_string()),
                    Err(err) => QueryResponse::status(err.to_string()),
                }
            }
        };

        Ok(to_json_binary(&resp)?)
    }
}

pub trait StorageRef {
    fn storage(&self) -> &dyn Storage;
}

impl StorageRef for ContextRef<'_> {
    fn storage(&self) -> &dyn Storage {
        self.deps.storage
    }
}

pub struct ContextMut<'a> {
    deps: DepsMut<'a>,
    env: Env,
}

impl<'a> ContextMut<'a> {
    pub fn new(deps: DepsMut<'a>, env: Env) -> Self {
        Self { deps, env }
    }

    pub fn env(&self) -> &Env {
        &self.env
    }

    pub fn log(&self, msg: &str) {
        self.deps.api.debug(msg)
    }

    pub fn client_id(&self) -> Result<ClientId, IdentifierError> {
        ClientId::from_str(self.env.contract.address.as_str())
    }

    pub fn instantiate(&mut self, msg: InstantiateMsg) -> Result<Binary, ContractError> {
        let client_id = self.client_id()?;

        let any = Any::decode(&mut msg.client_state.as_slice())?;

        let client_state = ClientState::try_from(any)?;

        let any_consensus_state = Any::decode(&mut msg.consensus_state.as_slice())?;

        client_state.initialise(self, &client_id, any_consensus_state)?;

        Ok(to_json_binary(&ContractResult::success())?)
    }

    pub fn execute(&mut self, msg: ExecuteMsg) -> Result<Binary, ContractError> {
        let client_id = self.client_id()?;

        let client_state = self.client_state(&client_id)?;

        let result = match msg {
            ExecuteMsg::VerifyMembership(msg) => {
                let msg = VerifyMembershipMsg::try_from(msg)?;

                let client_cons_state_path = ClientConsensusStatePath::new(
                    client_id.clone(),
                    msg.height.revision_number(),
                    msg.height.revision_height(),
                );

                let consensus_state = self.consensus_state(&client_cons_state_path)?;

                client_state
                    .verify_membership(
                        &msg.prefix,
                        &msg.proof,
                        consensus_state.root(),
                        msg.path,
                        msg.value,
                    )
                    .map_err(ContextError::from)?;

                ContractResult::success()
            }
            ExecuteMsg::VerifyNonMembership(msg) => {
                let msg = VerifyNonMembershipMsg::try_from(msg)?;

                let client_cons_state_path = ClientConsensusStatePath::new(
                    client_id.clone(),
                    msg.height.revision_number(),
                    msg.height.revision_height(),
                );

                let consensus_state = self.consensus_state(&client_cons_state_path)?;

                client_state.verify_non_membership(
                    &msg.prefix,
                    &msg.proof,
                    consensus_state.root(),
                    msg.path,
                )?;

                ContractResult::success()
            }
            ExecuteMsg::VerifyClientMessage(msg) => {
                let msg = VerifyClientMessageMsg::try_from(msg)?;

                let (update_kind, any_client_msg): (_, Any) = match msg.client_message {
                    ClientMessage::Header(header) => (UpdateKind::UpdateClient, (*header).into()),
                    ClientMessage::Misbehaviour(misbehaviour) => {
                        (UpdateKind::SubmitMisbehaviour, (*misbehaviour).into())
                    }
                };

                client_state.verify_client_message(
                    self,
                    &client_id,
                    any_client_msg,
                    &update_kind,
                )?;

                ContractResult::success()
            }
            ExecuteMsg::CheckForMisbehaviour(msg) => {
                let msg = CheckForMisbehaviourMsg::try_from(msg)?;

                let (update_kind, any_client_msg): (_, Any) = match msg.client_message {
                    ClientMessage::Header(header) => (UpdateKind::UpdateClient, (*header).into()),
                    ClientMessage::Misbehaviour(misbehaviour) => {
                        (UpdateKind::SubmitMisbehaviour, (*misbehaviour).into())
                    }
                };

                let result = client_state.check_for_misbehaviour(
                    self,
                    &client_id,
                    any_client_msg,
                    &update_kind,
                )?;

                ContractResult::success().misbehaviour(result)
            }
            ExecuteMsg::UpdateStateOnMisbehaviour(msg_raw) => {
                let msg: UpdateStateOnMisbehaviourMsg =
                    UpdateStateOnMisbehaviourMsg::try_from(msg_raw)?;

                let (update_kind, any_client_msg) = match msg.client_message {
                    ClientMessage::Header(header) => (UpdateKind::UpdateClient, (*header).into()),
                    ClientMessage::Misbehaviour(misbehaviour) => {
                        (UpdateKind::SubmitMisbehaviour, (*misbehaviour).into())
                    }
                };

                client_state.update_state_on_misbehaviour(
                    self,
                    &client_id,
                    any_client_msg,
                    &update_kind,
                )?;

                ContractResult::success()
            }
            ExecuteMsg::UpdateState(msg_raw) => {
                let msg = UpdateStateMsg::try_from(msg_raw)?;

                let (_, any_client_msg) = match msg.client_message {
                    ClientMessage::Header(header) => (UpdateKind::UpdateClient, (*header).into()),
                    ClientMessage::Misbehaviour(misbehaviour) => {
                        (UpdateKind::SubmitMisbehaviour, (*misbehaviour).into())
                    }
                };

                client_state.update_state(self, &client_id, any_client_msg)?;

                ContractResult::success()
            }
            ExecuteMsg::CheckSubstituteAndUpdateState(_) => {
                ContractResult::error("ibc-rs does no support this feature yet".to_string())
            }
            ExecuteMsg::VerifyUpgradeAndUpdateState(msg) => {
                let msg = VerifyUpgradeAndUpdateStateMsg::try_from(msg)?;

                let client_cons_state_path = ClientConsensusStatePath::new(
                    client_id.clone(),
                    client_state.latest_height().revision_number(),
                    client_state.latest_height().revision_height(),
                );

                let consensus_state = self.consensus_state(&client_cons_state_path)?;

                client_state.verify_upgrade_client(
                    msg.upgrade_client_state.clone(),
                    msg.upgrade_consensus_state.clone(),
                    msg.proof_upgrade_client,
                    msg.proof_upgrade_consensus_state,
                    consensus_state.root(),
                )?;

                client_state.update_state_on_upgrade(
                    self,
                    &client_id,
                    msg.upgrade_client_state,
                    msg.upgrade_consensus_state,
                )?;

                ContractResult::success()
            }
        };
        Ok(to_json_binary(&result)?)
    }
}

pub trait StorageMut: StorageRef {
    fn storage_mut(&mut self) -> &mut dyn Storage;
}

impl StorageRef for ContextMut<'_> {
    fn storage(&self) -> &dyn Storage {
        self.deps.storage
    }
}

impl StorageMut for ContextMut<'_> {
    fn storage_mut(&mut self) -> &mut dyn Storage {
        self.deps.storage
    }
}
