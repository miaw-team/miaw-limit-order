#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::order::{cancel_order, execute_order, submit_order};
use crate::query::{query_config, query_last_order_id, query_order, query_orders};
use crate::state::{Config, CONFIG, LAST_ORDER_ID};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        fee_token: deps.api.addr_validate(msg.fee_token.as_str())?,
        min_fee_amount: msg.min_fee_amount,
        terraswap_factory: deps.api.addr_validate(msg.terraswap_factory.as_str())?,
    };

    CONFIG.save(deps.storage, &config)?;
    LAST_ORDER_ID.save(deps.storage, &0u64)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::SubmitOrder {
            offer_asset,
            ask_asset,
            fee_amount,
        } => submit_order(deps, env, info, offer_asset, ask_asset, fee_amount),
        ExecuteMsg::CancelOrder { order_id } => cancel_order(deps, info, order_id),
        ExecuteMsg::ExecuteOrder { order_id } => execute_order(deps, info, order_id),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Order { order_id } => to_binary(&query_order(deps, order_id)?),
        QueryMsg::Orders {
            bidder_addr,
            start_after,
            limit,
            order_by,
        } => to_binary(&query_orders(
            deps,
            bidder_addr,
            start_after,
            limit,
            order_by,
        )?),
        QueryMsg::LastOrderId {} => to_binary(&query_last_order_id(deps)?),
    }
}
