use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use otcer_pkg::otcer::{
    definitions::Config,
    msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};
use rhaki_cw_plus::traits::{IntoAddr, IntoBinaryResult};

use crate::{
    execute::{run_cancel_otc, run_claim_otc, run_create_otc, run_execute_otc},
    query::{
        qy_active_position, qy_active_positions, qy_active_positions_by_dealer,
        qy_active_positions_by_owner, qy_executed_position, qy_executed_positions,
        qy_executed_positions_by_dealer, qy_executed_positions_by_owner,
    },
    response::ContractResponse,
    state::CONFIG,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResponse {
    let config = Config::new(
        deps.as_ref(),
        msg.owner.clone().into_addr(deps.api)?,
        msg.fee,
        msg.fee_collector.into_addr(deps.api)?,
    )?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", msg.owner))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ContractResponse {
    match msg {
        ExecuteMsg::CreateOtc(msg) => run_create_otc(deps, env, info, msg),
        ExecuteMsg::ExecuteOtc(msg) => run_execute_otc(deps, env, info, msg),
        ExecuteMsg::ClaimOtc(msg) => run_claim_otc(deps, env, info, msg),
        ExecuteMsg::CancelOtc(msg) => run_cancel_otc(deps, env, info, msg),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ActivePosition { id } => qy_active_position(deps, id).into_binary(),
        QueryMsg::ExecutedPosition { id } => qy_executed_position(deps, id).into_binary(),
        QueryMsg::ActivePositions { limit, start_after } => {
            qy_active_positions(deps, limit, start_after).into_binary()
        }
        QueryMsg::ExecutedPositions { limit, start_after } => {
            qy_executed_positions(deps, limit, start_after).into_binary()
        }
        QueryMsg::ActivePositionsByOwner {
            owner,
            limit,
            start_after,
        } => qy_active_positions_by_owner(deps, owner, limit, start_after).into_binary(),
        QueryMsg::ActrivePositionByDealer {
            dealer,
            limit,
            start_after,
        } => qy_active_positions_by_dealer(deps, dealer, limit, start_after).into_binary(),
        QueryMsg::ExecutedPositionsByOwner {
            owner,
            limit,
            start_after,
        } => qy_executed_positions_by_owner(deps, owner, limit, start_after).into_binary(),
        QueryMsg::ExecutedPositionBtDealer {
            dealer,
            limit,
            start_after,
        } => qy_executed_positions_by_dealer(deps, dealer, limit, start_after).into_binary(),
    }
}

#[entry_point]
pub fn migrate(_deps: Deps, _env: Env, _msg: MigrateMsg) -> ContractResponse {
    Ok(Response::default())
}
