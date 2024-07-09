use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError};
use otcer_pkg::otcer::{
    definitions::OtcPosition,
    msgs::{CancelOtcMsg, ClaimOtcMsg, CreateOtcMsg, ExecuteOtcMsg},
};
use rhaki_cw_plus::traits::IntoAddr;

use crate::{
    functions::{after_action, cancel_otc, collect_otc_items, send_fee, send_otc_items},
    response::{ContractError, ContractResponse},
    state::{positions, CONFIG},
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

    if !remaining_coins.is_empty() {
        return Err(ContractError::ExtraCoinReceived);
    }

    CONFIG.save(deps.storage, &config)?;

    positions().save(deps.storage, config.counter_otc, &position)?;

    Ok(Response::new()
        .add_messages(msgs_deposit)
        .add_attribute("action", "create_otc")
        .add_attribute(
            "executor",
            position
                .executor
                .unwrap_or("undefined".into_unchecked_addr()),
        )
        .add_attribute("otc_id", config.counter_otc.to_string()))
}

pub fn run_execute_otc(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteOtcMsg,
) -> ContractResponse {
    let mut position = positions().load(deps.storage, msg.id)?;
    position.active(&env, &info.sender)?;

    let config = CONFIG.load(deps.storage)?;

    let (msgs_deposit, remaining_coins) =
        collect_otc_items(&env, &position.ask, info.sender, info.funds)?;

    let msgs_fee_ask = send_fee(
        deps.as_ref(),
        &env,
        &mut position.ask,
        &config.variable_provider,
    )?;

    let msgs_fee_offer = send_fee(
        deps.as_ref(),
        &env,
        &mut position.offer,
        &config.variable_provider,
    )?;

    if !remaining_coins.is_empty() {
        return Err(ContractError::ExtraCoinReceived);
    }

    let msgs_to_owner = send_otc_items(&env, &mut position.ask, &position.status, &position.owner)?;
    let msgs_to_executor = send_otc_items(
        &env,
        &mut position.offer,
        &position.status,
        &position.executor.clone().unwrap(),
    )?;

    let attrs_close = after_action(deps, &env, &mut position)?;

    Ok(Response::new()
        .add_messages(msgs_deposit)
        .add_messages(msgs_fee_ask)
        .add_messages(msgs_fee_offer)
        .add_messages(msgs_to_owner)
        .add_messages(msgs_to_executor)
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
    let mut position = positions().load(deps.storage, msg.id)?;

    let msgs = if info.sender == position.owner {
        send_otc_items(&env, &mut position.ask, &position.status, &info.sender)?
    } else if info.sender == position.executor.clone().unwrap() {
        send_otc_items(&env, &mut position.offer, &position.status, &info.sender)?
    } else {
        return Err(ContractError::Unauthorized {});
    };

    if msgs.is_empty() {
        return Err(StdError::generic_err("Nothing to claim").into());
    }

    let attrs_close = after_action(deps, &env, &mut position)?;

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
    let position = positions().load(deps.storage, msg.id)?;

    if info.sender != position.owner {
        return Err(ContractError::Unauthorized {});
    }

    if !position.status.is_in_pending() {
        return Err(StdError::generic_err("Can't cancel a position non in pending status").into());
    }
    let msgs_to_owner = cancel_otc(&env, &position)?;

    positions().remove(deps.storage, msg.id)?;

    Ok(Response::new()
        .add_messages(msgs_to_owner)
        .add_attribute("action", "cancel_otc")
        .add_attribute("id", msg.id.to_string()))
}
