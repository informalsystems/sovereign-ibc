use alloc::str::FromStr;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use ibc::core::ics02_client::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation, UpdateKind,
};
use ibc::core::ics02_client::consensus_state::ConsensusState;
use ibc::core::ics02_client::ClientExecutionContext;
use ibc::core::ics24_host::identifier::ClientId;
use ibc::core::ics24_host::path::ClientConsensusStatePath;
use ibc::core::{ContextError, ValidationContext};
use ibc::proto::Any;
use ics11_sov_celestia::client_message::ClientMessage;

use crate::contexts::{ContextMut, ContextRef};
use crate::types::error::ContractError;
use crate::types::msgs::{
    CheckForMisbehaviourMsg, ExecuteMsg, ExportMetadataMsg, InstantiateMsg, QueryMsg, StatusMsg,
    UpdateStateMsg, UpdateStateOnMisbehaviourMsg, VerifyClientMessageMsg, VerifyMembershipMsg,
    VerifyNonMembershipMsg, VerifyUpgradeAndUpdateStateMsg,
};
use crate::types::processed_states::ReadonlyProcessedStates;
use crate::types::response::{ContractResult, QueryResponse};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<'_>,
    env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut ctx = ContextMut::new(deps, env);

    let client_id = ClientId::from_str("08-wasm-0").expect("client id is valid");

    let client_state = ctx.client_state(&client_id)?;

    ctx.store_update_height(
        client_id.clone(),
        client_state.latest_height(),
        ctx.host_height()?,
    )?;

    ctx.store_update_time(
        client_id,
        client_state.latest_height(),
        ctx.host_timestamp()?,
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<'_>,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let mut ctx = ContextMut::new(deps, env);

    let client_id = ClientId::from_str("08-wasm-0").expect("client id is valid");

    let data = process_message(&mut ctx, client_id, msg)?;

    let mut response = Response::default();

    response.data = Some(data);

    Ok(response)
}

fn process_message(
    ctx: &mut ContextMut<'_>,
    client_id: ClientId,
    msg: ExecuteMsg,
) -> Result<Binary, ContractError> {
    let result = match msg {
        ExecuteMsg::VerifyMembership(msg) => {
            let msg = VerifyMembershipMsg::try_from(msg)?;

            let client_state = ctx.client_state(&client_id)?;

            let client_cons_state_path = ClientConsensusStatePath::new(&client_id, &msg.height);

            let consensus_state = ctx.consensus_state(&client_cons_state_path)?;

            client_state
                .verify_membership(
                    &msg.prefix,
                    &msg.proof,
                    consensus_state.root(),
                    msg.path,
                    msg.value,
                )
                .map_err(ContextError::from)?;

            to_binary(&ContractResult::success())
        }
        ExecuteMsg::VerifyNonMembership(msg) => {
            let msg = VerifyNonMembershipMsg::try_from(msg)?;

            let client_state = ctx.client_state(&client_id)?;

            let client_cons_state_path = ClientConsensusStatePath::new(&client_id, &msg.height);

            let consensus_state = ctx.consensus_state(&client_cons_state_path)?;

            client_state
                .verify_non_membership(&msg.prefix, &msg.proof, consensus_state.root(), msg.path)
                .map_err(ContextError::from)?;

            to_binary(&ContractResult::success())
        }
        ExecuteMsg::VerifyClientMessage(msg) => {
            let msg = VerifyClientMessageMsg::try_from(msg)?;

            let client_state = ctx.client_state(&client_id)?;

            let (update_kind, any_client_msg): (_, Any) = match msg.client_message {
                ClientMessage::Header(header) => (UpdateKind::UpdateClient, (*header).into()),
                ClientMessage::Misbehaviour(misbehaviour) => {
                    (UpdateKind::SubmitMisbehaviour, (*misbehaviour).into())
                }
            };

            client_state
                .verify_client_message(ctx, &client_id, any_client_msg, &update_kind)
                .map_err(ContextError::from)?;

            to_binary(&ContractResult::success())
        }
        ExecuteMsg::CheckForMisbehaviour(msg) => {
            let msg = CheckForMisbehaviourMsg::try_from(msg)?;

            let client_state = ctx.client_state(&client_id)?;

            let (update_kind, any_client_msg): (_, Any) = match msg.client_message {
                ClientMessage::Header(header) => (UpdateKind::UpdateClient, (*header).into()),
                ClientMessage::Misbehaviour(misbehaviour) => {
                    (UpdateKind::SubmitMisbehaviour, (*misbehaviour).into())
                }
            };

            let result = client_state
                .check_for_misbehaviour(ctx, &client_id, any_client_msg, &update_kind)
                .map_err(ContextError::from)?;

            to_binary(&ContractResult::success().misbehaviour(result))
        }
        ExecuteMsg::UpdateStateOnMisbehaviour(msg_raw) => {
            let msg = UpdateStateOnMisbehaviourMsg::try_from(msg_raw)?;

            let client_state = ctx.client_state(&client_id)?;

            let (update_kind, any_client_msg) = match msg.client_message {
                ClientMessage::Header(header) => (UpdateKind::UpdateClient, (*header).into()),
                ClientMessage::Misbehaviour(misbehaviour) => {
                    (UpdateKind::SubmitMisbehaviour, (*misbehaviour).into())
                }
            };

            client_state
                .update_state_on_misbehaviour(ctx, &client_id, any_client_msg, &update_kind)
                .map_err(ContextError::from)?;

            to_binary(&ContractResult::success())
        }
        ExecuteMsg::UpdateState(msg_raw) => {
            let msg = UpdateStateMsg::try_from(msg_raw)?;

            let client_state = ctx.client_state(&client_id)?;

            let (_, any_client_msg) = match msg.client_message {
                ClientMessage::Header(header) => (UpdateKind::UpdateClient, (*header).into()),
                ClientMessage::Misbehaviour(misbehaviour) => {
                    (UpdateKind::SubmitMisbehaviour, (*misbehaviour).into())
                }
            };

            client_state
                .update_state(ctx, &client_id, any_client_msg)
                .map_err(ContextError::from)?;

            to_binary(&ContractResult::success())
        }
        ExecuteMsg::CheckSubstituteAndUpdateState(_) => to_binary(&ContractResult::error(
            "ibc-rs does no support this feature yet".to_string(),
        )),
        ExecuteMsg::VerifyUpgradeAndUpdateState(msg) => {
            let msg = VerifyUpgradeAndUpdateStateMsg::try_from(msg)?;

            let old_client_state = ctx.client_state(&client_id)?;

            let client_cons_state_path =
                ClientConsensusStatePath::new(&client_id, &old_client_state.latest_height());

            let consensus_state = ctx.consensus_state(&client_cons_state_path)?;

            old_client_state
                .verify_upgrade_client(
                    msg.upgrade_client_state.clone(),
                    msg.upgrade_consensus_state.clone(),
                    msg.proof_upgrade_client,
                    msg.proof_upgrade_consensus_state,
                    consensus_state.root(),
                )
                .map_err(ContextError::from)?;

            old_client_state
                .update_state_on_upgrade(
                    ctx,
                    &client_id,
                    msg.upgrade_client_state,
                    msg.upgrade_consensus_state,
                )
                .map_err(ContextError::from)?;

            to_binary(&ContractResult::success())
        }
    };
    Ok(result?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<'_>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let ctx = ContextRef::new(deps, env);

    let client_id = ClientId::from_str("08-wasm-0").expect("client id is valid");

    match msg {
        QueryMsg::ClientTypeMsg(_) => unimplemented!("ClientTypeMsg"),
        QueryMsg::GetLatestHeightsMsg(_) => unimplemented!("GetLatestHeightsMsg"),
        QueryMsg::ExportMetadata(ExportMetadataMsg {}) => {
            let ro_proceeded_state = ReadonlyProcessedStates::new(deps.storage);
            to_binary(&QueryResponse::genesis_metadata(
                ro_proceeded_state.get_metadata(),
            ))
        }
        QueryMsg::Status(StatusMsg {}) => {
            let client_state = match ctx.client_state(&client_id) {
                Ok(cs) => cs,
                Err(_) => return to_binary(&QueryResponse::status("Client not found".to_string())),
            };

            match client_state.status(&ctx, &client_id) {
                Ok(status) => to_binary(&QueryResponse::status(status.to_string())),
                Err(err) => to_binary(&QueryResponse::status(err.to_string())),
            }
        }
    }
}
