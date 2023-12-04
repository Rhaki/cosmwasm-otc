use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError};
use otcer_pkg::otcer::{
    definitions::OtcPosition,
    msgs::{CancelOtcMsg, ClaimOtcMsg, CreateOtcMsg, ExecuteOtcMsg},
};
use rhaki_cw_plus::traits::IntoAddr;

use crate::{
    functions::{cancel_otc, collect_otc_items, send_fee, send_otc_items, try_close_position},
    response::{ContractError, ContractResponse},
    state::{active_positions, CONFIG},
};

pub fn run_create_otc(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CreateOtcMsg,
) -> ContractResponse {
    let mut config = CONFIG.load(deps.storage)?;
    config.counter_otc += 1;

    let position = OtcPosition::from_create_otc_msg(
        deps.as_ref(),
        &env,
        msg,
        config.counter_otc,
        info.sender.clone(),
    )?;
    position.validate(deps.as_ref())?;

    let (msgs_deposit, remaining_coins) =
        collect_otc_items(&env, &position.offer, info.sender, info.funds)?;

    let msgs_fee = send_fee(&env, &config.fee, &config.fee_collector, remaining_coins)?;

    CONFIG.save(deps.storage, &config)?;

    active_positions().save(deps.storage, config.counter_otc, &position)?;

    Ok(Response::new()
        .add_messages(msgs_deposit)
        .add_messages(msgs_fee)
        .add_attribute("action", "create_orc")
        .add_attribute(
            "dealer",
            position.dealer.unwrap_or("undefined".into_unchecked_addr()),
        )
        .add_attribute("otc_id", config.counter_otc.to_string()))
}

pub fn run_execute_otc(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteOtcMsg,
) -> ContractResponse {
    let mut position = active_positions().load(deps.storage, msg.id)?;
    position.active(&env, &info.sender)?;

    let config = CONFIG.load(deps.storage)?;

    let (msgs_deposit, remaining_coins) =
        collect_otc_items(&env, &position.ask, info.sender, info.funds)?;

    let msgs_fee = send_fee(&env, &config.fee, &config.fee_collector, remaining_coins)?;

    let msgs_to_owner = send_otc_items(&env, &mut position.ask, &position.status, &position.owner)?;
    let msgs_to_dealer = send_otc_items(
        &env,
        &mut position.offer,
        &position.status,
        &position.dealer.clone().unwrap(),
    )?;

    let attrs_close = try_close_position(deps, &env, &mut position)?;

    Ok(Response::new()
        .add_messages(msgs_deposit)
        .add_messages(msgs_fee)
        .add_messages(msgs_to_owner)
        .add_messages(msgs_to_dealer)
        .add_attribute("action", "execute_otc")
        .add_attribute("otc_id", msg.id.to_string())
        .add_attributes(attrs_close))
}

pub fn run_claim_otc(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ClaimOtcMsg,
) -> ContractResponse {
    let mut position = active_positions().load(deps.storage, msg.id)?;

    let msgs = if info.sender == position.owner {
        send_otc_items(&env, &mut position.ask, &position.status, &info.sender)?
    } else if info.sender == position.dealer.clone().unwrap() {
        send_otc_items(&env, &mut position.offer, &position.status, &info.sender)?
    } else {
        return Err(ContractError::Unauthorized {});
    };

    if msgs.is_empty() {
        return Err(StdError::generic_err("Nothing to claim").into());
    }

    let attrs_close = try_close_position(deps, &env, &mut position)?;

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "claim")
        .add_attribute("id", msg.id.to_string())
        .add_attributes(attrs_close))
}

pub fn run_cancel_otc(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CancelOtcMsg,
) -> ContractResponse {
    let position = active_positions().load(deps.storage, msg.id)?;

    if info.sender != position.owner {
        return Err(ContractError::Unauthorized {});
    }

    if !position.status.is_in_pending() {
        return Err(StdError::generic_err("Can't cancel a position non in pending status").into());
    }
    let msgs_to_owner = cancel_otc(&env, &position)?;

    active_positions().remove(deps.storage, msg.id)?;

    Ok(Response::new()
        .add_messages(msgs_to_owner)
        .add_attribute("action", "cancel_otc")
        .add_attribute("id", msg.id.to_string()))
}
