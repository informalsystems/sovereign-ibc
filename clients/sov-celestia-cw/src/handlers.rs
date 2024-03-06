use cosmwasm_std::{to_json_binary, Binary};
use ibc_core::client::context::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation,
};
use ibc_core::client::context::consensus_state::ConsensusState as _;
use ibc_core::client::context::ClientValidationContext;
use ibc_core::host::types::path::ClientConsensusStatePath;
use ibc_core::primitives::proto::Any;
use prost::Message;
use sov_celestia_client::client_state::ClientState;
use sov_celestia_client::types::client_message::ClientMessage;

use crate::context::Context;
use crate::types::{
    CheckForMisbehaviourMsg, ContractError, ContractResult, ExportMetadataMsg, InstantiateMsg,
    QueryMsg, QueryResponse, StatusMsg, SudoMsg, UpdateStateMsg, UpdateStateOnMisbehaviourMsg,
    VerifyClientMessageMsg, VerifyMembershipMsg, VerifyNonMembershipMsg,
    VerifyUpgradeAndUpdateStateMsg,
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

    pub fn sudo(&mut self, msg: SudoMsg) -> Result<Binary, ContractError> {
        let client_id = self.client_id();

        let client_state = self.client_state(&client_id)?;

        let result = match msg {
            SudoMsg::UpdateState(msg_raw) => {
                let msg = UpdateStateMsg::try_from(msg_raw)?;

                let any_client_msg = match msg.client_message {
                    ClientMessage::Header(header) => (*header).into(),
                    ClientMessage::Misbehaviour(misbehaviour) => (*misbehaviour).into(),
                };

                let heights = client_state.update_state(self, &client_id, any_client_msg)?;

                ContractResult::success().heights(heights)
            }
            SudoMsg::UpdateStateOnMisbehaviour(msg_raw) => {
                let msg: UpdateStateOnMisbehaviourMsg =
                    UpdateStateOnMisbehaviourMsg::try_from(msg_raw)?;

                let any_client_msg = match msg.client_message {
                    ClientMessage::Header(header) => (*header).into(),
                    ClientMessage::Misbehaviour(misbehaviour) => (*misbehaviour).into(),
                };

                client_state.update_state_on_misbehaviour(self, &client_id, any_client_msg)?;

                ContractResult::success()
            }
            SudoMsg::VerifyMembership(msg) => {
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
            SudoMsg::VerifyNonMembership(msg) => {
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
            SudoMsg::VerifyUpgradeAndUpdateState(msg) => {
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
            SudoMsg::MigrateClientStore(_) => {
                return Err(ContractError::InvalidMsg(
                    "ibc-rs does no support this feature yet".to_string(),
                ));
            }
        };
        Ok(to_json_binary(&result)?)
    }

    pub fn query(&self, msg: QueryMsg) -> Result<Binary, ContractError> {
        let client_id = self.client_id();

        let client_state = self.client_state(&client_id)?;

        let resp = match msg {
            QueryMsg::Status(StatusMsg {}) => match client_state.status(self, &client_id) {
                Ok(status) => QueryResponse::success().status(status.to_string()),
                Err(err) => QueryResponse::success().status(err.to_string()),
            },
            QueryMsg::ExportMetadata(ExportMetadataMsg {}) => {
                QueryResponse::success().genesis_metadata(self.get_metadata()?)
            }
            QueryMsg::TimestampAtHeight(msg) => {
                let client_cons_state_path = ClientConsensusStatePath::new(
                    client_id,
                    msg.height.revision_number(),
                    msg.height.revision_height(),
                );

                let consensus_state = self.consensus_state(&client_cons_state_path)?;

                QueryResponse::success().timestamp(consensus_state.timestamp().nanoseconds())
            }
            QueryMsg::VerifyClientMessage(msg) => {
                let msg = VerifyClientMessageMsg::try_from(msg)?;

                let any_client_msg: Any = match msg.client_message {
                    ClientMessage::Header(header) => (*header).into(),
                    ClientMessage::Misbehaviour(misbehaviour) => (*misbehaviour).into(),
                };

                client_state.verify_client_message(self, &client_id, any_client_msg)?;

                QueryResponse::success()
            }
            QueryMsg::CheckForMisbehaviour(msg) => {
                let msg = CheckForMisbehaviourMsg::try_from(msg)?;

                let any_client_msg: Any = match msg.client_message {
                    ClientMessage::Header(header) => (*header).into(),
                    ClientMessage::Misbehaviour(misbehaviour) => (*misbehaviour).into(),
                };

                let result =
                    client_state.check_for_misbehaviour(self, &client_id, any_client_msg)?;

                QueryResponse::success().misbehaviour(result)
            }
        };

        Ok(to_json_binary(&resp)?)
    }
}
