use cosmwasm_std::{Deps, Order, StdResult};
use otcer_pkg::otc::definitions::OtcPosition;

use crate::state::{active_positions, execute_positions};

pub fn qy_active_position(deps: Deps, id: u64) -> StdResult<OtcPosition> {
    active_positions().load(deps.storage, id)
}

pub fn qy_executed_position(deps: Deps, id: u64) -> StdResult<OtcPosition> {
    execute_positions().load(deps.storage, id)
}

pub fn qy_active_positions(
    deps: Deps,
    limit: Option<u32>,
    start_after: Option<u64>,
) -> StdResult<Vec<OtcPosition>> {
    rhaki_cw_plus::storage::multi_index::get_items(
        deps.storage,
        active_positions(),
        Order::Ascending,
        limit,
        start_after,
    )
    .map(|val| val.into_iter().map(|(_, val)| val).collect())
}

pub fn qy_executed_positions(
    deps: Deps,
    limit: Option<u32>,
    start_after: Option<u64>,
) -> StdResult<Vec<OtcPosition>> {
    rhaki_cw_plus::storage::multi_index::get_items(
        deps.storage,
        execute_positions(),
        Order::Ascending,
        limit,
        start_after,
    )
    .map(|val| val.into_iter().map(|(_, val)| val).collect())
}

pub fn qy_active_positions_by_owner(
    deps: Deps,
    owner: String,
    limit: Option<u32>,
    start_after: Option<u64>,
) -> StdResult<Vec<OtcPosition>> {
    rhaki_cw_plus::storage::multi_index::get_multi_index_values(
        deps.storage,
        owner,
        active_positions().idx.owner,
        Order::Ascending,
        start_after,
        limit,
    )
    .map(|val| val.into_iter().map(|(_, val)| val).collect())
}

pub fn qy_active_positions_by_dealer(
    deps: Deps,
    dealer: String,
    limit: Option<u32>,
    start_after: Option<u64>,
) -> StdResult<Vec<OtcPosition>> {
    rhaki_cw_plus::storage::multi_index::get_multi_index_values(
        deps.storage,
        dealer,
        active_positions().idx.dealer,
        Order::Ascending,
        start_after,
        limit,
    )
    .map(|val| val.into_iter().map(|(_, val)| val).collect())
}

pub fn qy_executed_positions_by_owner(
    deps: Deps,
    owner: String,
    limit: Option<u32>,
    start_after: Option<u64>,
) -> StdResult<Vec<OtcPosition>> {
    rhaki_cw_plus::storage::multi_index::get_multi_index_values(
        deps.storage,
        owner,
        execute_positions().idx.owner,
        Order::Ascending,
        start_after,
        limit,
    )
    .map(|val| val.into_iter().map(|(_, val)| val).collect())
}

pub fn qy_executed_positions_by_dealer(
    deps: Deps,
    dealer: String,
    limit: Option<u32>,
    start_after: Option<u64>,
) -> StdResult<Vec<OtcPosition>> {
    rhaki_cw_plus::storage::multi_index::get_multi_index_values(
        deps.storage,
        dealer,
        execute_positions().idx.dealer,
        Order::Ascending,
        start_after,
        limit,
    )
    .map(|val| val.into_iter().map(|(_, val)| val).collect())
}
