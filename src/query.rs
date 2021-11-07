use cosmwasm_std::{Deps, StdResult};

use crate::{
    msg::{ConfigResponse, LastOrderIdResponse, OrderBy, OrderResponse, OrdersResponse},
    state::{read_orders, read_orders_by_user, Config, OrderInfo, CONFIG, LAST_ORDER_ID, ORDERS},
};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = CONFIG.load(deps.storage)?;

    config.as_res()
}

pub fn query_order(deps: Deps, order_id: u64) -> StdResult<OrderResponse> {
    let order: OrderInfo = ORDERS.load(deps.storage, &order_id.to_be_bytes())?;

    order.as_res()
}

pub fn query_orders(
    deps: Deps,
    bidder_addr: Option<String>,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<OrdersResponse> {
    let orders: Vec<OrderInfo> = if let Some(bidder_addr) = bidder_addr {
        read_orders_by_user(
            deps.storage,
            &deps.api.addr_validate(&bidder_addr)?,
            start_after,
            limit,
            order_by,
        )?
    } else {
        read_orders(deps.storage, start_after, limit, order_by)?
    };

    let resp = OrdersResponse {
        orders: orders
            .iter()
            .map(|order| order.as_res())
            .collect::<StdResult<Vec<OrderResponse>>>()?,
    };

    Ok(resp)
}

pub fn query_last_order_id(deps: Deps) -> StdResult<LastOrderIdResponse> {
    let last_order_id = LAST_ORDER_ID.load(deps.storage)?;

    Ok(LastOrderIdResponse { last_order_id })
}
