#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

use crate::context::Context;
use crate::types::{ContractError, InstantiateMsg, QueryMsg, SudoMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<'_>,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut ctx = Context::new_mut(deps, env)?;

    let data = ctx.instantiate(msg)?;

    Ok(Response::default().set_data(data))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<'_>,
    env: Env,
    _info: MessageInfo,
    msg: SudoMsg,
) -> Result<Response, ContractError> {
    let mut ctx = Context::new_mut(deps, env)?;

    let data = ctx.execute(msg)?;

    Ok(Response::default().set_data(data))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<'_>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let ctx = Context::new_ref(deps, env)?;

    ctx.query(msg)
        .map_err(|e| StdError::generic_err(e.to_string()))
}
