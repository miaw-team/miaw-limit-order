use cw_storage_plus::{Bound, Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Order, StdResult, Storage, Uint128};
use terraswap::asset::Asset;

use crate::msg::{ConfigResponse, OrderBy, OrderResponse};

pub const CONFIG: Item<Config> = Item::new("config");
pub const LAST_ORDER_ID: Item<u64> = Item::new("last_order_id");
pub const ORDERS: Map<&[u8], OrderInfo> = Map::new("orders");
pub const ORDERS_BY_USER: Map<(&[u8], &[u8]), bool> = Map::new("orders_by_user");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub fee_token: Addr,
    pub min_fee_amount: Uint128,
    pub terraswap_factory: Addr,
}

impl Config {
    pub fn as_res(&self) -> StdResult<ConfigResponse> {
        let res = ConfigResponse {
            fee_token: self.fee_token.to_string(),
            min_fee_amount: self.min_fee_amount,
            terraswap_factory: self.terraswap_factory.to_string(),
        };
        Ok(res)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderInfo {
    pub order_id: u64,
    pub bidder_addr: Addr,
    pub pair_addr: Addr,
    pub offer_asset: Asset,
    pub ask_asset: Asset,
    pub fee_amount: Uint128,
}

impl OrderInfo {
    pub fn as_res(&self) -> StdResult<OrderResponse> {
        let res = OrderResponse {
            order_id: self.order_id,
            bidder_addr: self.bidder_addr.to_string(),
            pair_addr: self.pair_addr.to_string(),
            offer_asset: self.offer_asset.clone(),
            ask_asset: self.ask_asset.clone(),
            fee_amount: self.fee_amount,
        };
        Ok(res)
    }
}

pub fn store_new_order(storage: &mut dyn Storage, order: &mut OrderInfo) -> StdResult<()> {
    let new_id: u64 = LAST_ORDER_ID.load(storage)? + 1u64;
    order.order_id = new_id;

    ORDERS.save(storage, &new_id.to_be_bytes(), order)?;
    ORDERS_BY_USER.save(
        storage,
        (order.bidder_addr.as_bytes(), &new_id.to_be_bytes()),
        &true,
    )?;
    LAST_ORDER_ID.save(storage, &new_id)?;

    Ok(())
}

pub fn remove_order(storage: &mut dyn Storage, order: &OrderInfo) {
    ORDERS.remove(storage, &order.order_id.to_be_bytes());
    ORDERS_BY_USER.remove(
        storage,
        (order.bidder_addr.as_bytes(), &order.order_id.to_be_bytes()),
    );
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn read_orders_by_user(
    storage: &dyn Storage,
    user: &Addr,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<Vec<OrderInfo>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let (start, end, order_by) = match order_by {
        Some(OrderBy::Asc) => (
            calc_range_start(start_after).map(Bound::exclusive),
            None,
            Order::Ascending,
        ),
        _ => (
            None,
            calc_range_end(start_after).map(Bound::exclusive),
            Order::Descending,
        ),
    };

    ORDERS_BY_USER
        .prefix(user.as_bytes())
        .range(storage, start, end, order_by)
        .take(limit)
        .map(|item| {
            let (k, _) = item?;
            ORDERS.load(storage, &k)
        })
        .collect()
}

pub fn read_orders(
    storage: &dyn Storage,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<Vec<OrderInfo>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let (start, end, order_by) = match order_by {
        Some(OrderBy::Asc) => (
            calc_range_start(start_after).map(Bound::exclusive),
            None,
            Order::Ascending,
        ),
        _ => (
            None,
            calc_range_end(start_after).map(Bound::exclusive),
            Order::Descending,
        ),
    };

    ORDERS
        .range(storage, start, end, order_by)
        .take(limit)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect()
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start(start_after: Option<u64>) -> Option<Vec<u8>> {
    start_after.map(|id| {
        let mut v = id.to_be_bytes().to_vec();
        v.push(1);
        v
    })
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_end(start_after: Option<u64>) -> Option<Vec<u8>> {
    start_after.map(|id| id.to_be_bytes().to_vec())
}
