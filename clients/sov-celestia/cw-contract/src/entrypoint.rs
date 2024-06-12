use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use ibc_client_cw::context::Context;
use ibc_client_cw::types::{ContractError, InstantiateMsg, QueryMsg, SudoMsg};

use crate::client_type::SovTmClient;

pub type SovTmContext<'a> = Context<'a, SovTmClient>;

#[entry_point]
pub fn instantiate(
    deps: DepsMut<'_>,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut ctx = SovTmContext::new_mut(deps, env)?;

    let data = ctx.instantiate(msg)?;

    Ok(Response::default().set_data(data))
}

#[entry_point]
pub fn sudo(deps: DepsMut<'_>, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    let mut ctx = SovTmContext::new_mut(deps, env)?;

    let data = ctx.sudo(msg)?;

    Ok(Response::default().set_data(data))
}

#[entry_point]
pub fn query(deps: Deps<'_>, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let ctx = SovTmContext::new_ref(deps, env)?;

    ctx.query(msg)
}
