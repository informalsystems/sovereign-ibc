#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};
use ibc_client_cw::context::Context;
use ibc_client_cw::types::{ContractError, InstantiateMsg, QueryMsg, SudoMsg};

use crate::client_type::SovTmClient;

pub type SovTmContext<'a> = Context<'a, SovTmClient>;

#[cfg_attr(not(feature = "library"), entry_point)]
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut<'_>, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    let mut ctx = SovTmContext::new_mut(deps, env)?;

    let data = ctx.sudo(msg)?;

    Ok(Response::default().set_data(data))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<'_>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let ctx = SovTmContext::new_ref(deps, env)?;

    ctx.query(msg)
        .map_err(|e| StdError::generic_err(e.to_string()))
}
