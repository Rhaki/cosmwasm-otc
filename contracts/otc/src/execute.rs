use cosmwasm_otc_pkg::otc::{
    definitions::{OtcItemsChecker, OtcPosition},
    msgs::{CancelOtcMsg, CreateOtcMsg, ExecuteOtcMsg},
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use rhaki_cw_plus::traits::IntoAddr;

use crate::{
    response::{ContractError, ContractResponse},
    state::{active_positions, execute_positions, CONFIG},
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
        msg,
        config.counter_otc,
        info.sender.clone(),
    )?;
    position.validate(deps.as_ref())?;

    let (msgs_deposit, remaining_coins) =
        position
            .ask
            .gather_items(env.contract.address, info.sender.clone(), Some(info.funds))?;

    let (msgs_fee, _) = config.fee.gather_items(
        config.fee_collector.clone(),
        info.sender,
        Some(remaining_coins),
    )?;

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
    let position = active_positions().load(deps.storage, msg.id)?;
    let config = CONFIG.load(deps.storage)?;

    if let Some(dealer) = position.dealer.clone() {
        if dealer != info.sender {
            return Err(ContractError::Unauthorized {});
        }
    }

    let (msgs_to_owner, remaining_funds) = position.offer.gather_items(
        position.owner.clone(),
        info.sender.clone(),
        Some(info.funds),
    )?;

    let (msg_fee, _) = config.fee.gather_items(
        config.fee_collector,
        info.sender.clone(),
        Some(remaining_funds),
    )?;

    let (msg_to_dealer, _) = position
        .ask
        .gather_items(info.sender, env.contract.address, None)?;

    active_positions().remove(deps.storage, msg.id)?;
    execute_positions().save(deps.storage, msg.id, &position)?;

    Ok(Response::new()
        .add_messages(msgs_to_owner)
        .add_messages(msg_fee)
        .add_messages(msg_to_dealer)
        .add_attribute("action", "execute_otc")
        .add_attribute("otc_id", msg.id.to_string()))
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

    let (msgs, _) = position
        .offer
        .gather_items(info.sender, env.contract.address, None)?;

    active_positions().remove(deps.storage, msg.id)?;

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "cancel_otc")
        .add_attribute("id", msg.id.to_string()))
}
