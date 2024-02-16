use cosmwasm_std::{to_json_binary, Binary};
use ibc_core::client::context::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation,
};
use ibc_core::client::context::consensus_state::ConsensusState as _;
use ibc_core::client::types::UpdateKind;
use ibc_core::host::types::path::ClientConsensusStatePath;
use ibc_core::host::ValidationContext;
use ibc_proto::google::protobuf::Any;
use prost::Message;
use sov_celestia_client::client_state::ClientState;
use sov_celestia_client::types::client_message::ClientMessage;

use crate::context::Context;
use crate::types::{
    CheckForMisbehaviourMsg, ContractError, ContractResult, ExecuteMsg, ExportMetadataMsg,
    InstantiateMsg, QueryMsg, QueryResponse, StatusMsg, UpdateStateMsg,
    UpdateStateOnMisbehaviourMsg, VerifyClientMessageMsg, VerifyMembershipMsg,
    VerifyNonMembershipMsg, VerifyUpgradeAndUpdateStateMsg,
};

impl<'a> Context<'a> {
    pub fn instantiate(&mut self, msg: InstantiateMsg) -> Result<Binary, ContractError> {
        let any = Any::decode(&mut msg.client_state.as_slice())?;

        let client_state = ClientState::try_from(any)?;

        let any_consensus_state = Any::decode(&mut msg.consensus_state.as_slice())?;

        self.set_checksum(msg.checksum);

        client_state.initialise(self, &self.client_id(), any_consensus_state)?;

        Ok(to_json_binary(&ContractResult::success())?)
    }

    pub fn execute(&mut self, msg: ExecuteMsg) -> Result<Binary, ContractError> {
        let client_id = self.client_id();

        let client_state = self.client_state(&client_id)?;

        let result = match msg {
            ExecuteMsg::VerifyMembership(msg) => {
                let msg = VerifyMembershipMsg::try_from(msg)?;

                let client_cons_state_path = ClientConsensusStatePath::new(
                    self.client_id(),
                    msg.height.revision_number(),
                    msg.height.revision_height(),
                );

                let consensus_state = self.consensus_state(&client_cons_state_path)?;

                client_state.verify_membership(
                    &msg.prefix,
                    &msg.proof,
                    consensus_state.root(),
                    msg.path,
                    msg.value,
                )?;

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

                let any_client_msg: Any = match msg.client_message {
                    ClientMessage::Header(header) => (*header).into(),
                    ClientMessage::Misbehaviour(misbehaviour) => (*misbehaviour).into(),
                };

                client_state.verify_client_message(self, &client_id, any_client_msg)?;

                ContractResult::success()
            }
            ExecuteMsg::CheckForMisbehaviour(msg) => {
                let msg = CheckForMisbehaviourMsg::try_from(msg)?;

                let any_client_msg: Any = match msg.client_message {
                    ClientMessage::Header(header) => (*header).into(),
                    ClientMessage::Misbehaviour(misbehaviour) => (*misbehaviour).into(),
                };

                let result =
                    client_state.check_for_misbehaviour(self, &client_id, any_client_msg)?;

                ContractResult::success().misbehaviour(result)
            }
            ExecuteMsg::UpdateStateOnMisbehaviour(msg_raw) => {
                let msg: UpdateStateOnMisbehaviourMsg =
                    UpdateStateOnMisbehaviourMsg::try_from(msg_raw)?;

                let any_client_msg = match msg.client_message {
                    ClientMessage::Header(header) => (*header).into(),
                    ClientMessage::Misbehaviour(misbehaviour) => (*misbehaviour).into(),
                };

                client_state.update_state_on_misbehaviour(self, &client_id, any_client_msg)?;

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

    pub fn query(&self, msg: QueryMsg) -> Result<Binary, ContractError> {
        let client_id = self.client_id();

        let resp = match msg {
            QueryMsg::ClientTypeMsg(_) => unimplemented!("ClientTypeMsg"),
            QueryMsg::GetLatestHeightsMsg(_) => unimplemented!("GetLatestHeightsMsg"),
            QueryMsg::ExportMetadata(ExportMetadataMsg {}) => {
                QueryResponse::genesis_metadata(self.get_metadata()?)
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
