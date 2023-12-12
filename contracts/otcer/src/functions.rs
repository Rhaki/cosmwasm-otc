use cosmwasm_std::{
    attr, Addr, Attribute, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, StdError, StdResult,
    Uint128,
};
use otcer_pkg::{
    otcer::definitions::{OtcItem, OtcItemInfo, OtcPosition, OtcPositionStatus},
    variable_provider::vp_get_fee_and_collector,
};

use crate::state::positions;

pub fn collect_otc_items(
    env: &Env,
    items: &Vec<OtcItem>,
    sender: Addr,
    funds: Vec<Coin>,
) -> StdResult<(Vec<CosmosMsg>, Vec<Coin>)> {
    let coins = assert_received_funds(
        &items.iter().map(|val| val.item_info.clone()).collect(),
        funds,
    )?;
    let mut msgs: Vec<CosmosMsg> = vec![];
    for item in items {
        match &item.item_info {
            OtcItemInfo::Cw20 { .. } | &OtcItemInfo::Cw721 { .. } => msgs.push(
                item.item_info
                    .build_send_msg(env, &sender, &env.contract.address, None)?,
            ),
            _ => {}
        }
    }

    Ok((msgs, coins))
}

pub fn send_otc_items(
    env: &Env,
    items: &mut Vec<OtcItem>,
    position_status: &OtcPositionStatus,
    to: &Addr,
) -> StdResult<Vec<CosmosMsg>> {
    let mut msgs: Vec<CosmosMsg> = vec![];
    for item in items {
        let amount = item.sendable_amount_and_update_claimed_amount(env, position_status)?;

        if amount > Uint128::zero() {
            msgs.push(item.item_info.build_send_msg(
                env,
                &env.contract.address,
                to,
                Some(amount),
            )?)
        }
    }
    Ok(msgs)
}

pub fn send_fee(
    deps: Deps,
    env: &Env,
    items_info: &mut Vec<OtcItem>,
    vp_addr: &Addr,
) -> StdResult<Vec<CosmosMsg>> {
    let (fee, collector) = vp_get_fee_and_collector(deps, vp_addr)?;

    if fee > Decimal::zero() {
        items_info
            .iter_mut()
            .filter_map(|val| {
                let fee_amount = val.item_info.get_elegible_fee_amount() * fee;

                if fee_amount > Uint128::zero() {
                    val.item_info.subtract_fee_amount(fee_amount).unwrap();

                    Some(val.item_info.build_send_msg(
                        env,
                        &env.contract.address,
                        &collector,
                        Some(fee_amount),
                    ))
                } else {
                    None
                }
            })
            .collect()
    } else {
        Ok(vec![])
    }
}

pub fn cancel_otc(env: &Env, position: &OtcPosition) -> StdResult<Vec<CosmosMsg>> {
    build_send_otc_info_items(
        env,
        &position
            .offer
            .iter()
            .map(|val| val.item_info.clone())
            .collect(),
        &position.owner,
    )
}

pub fn build_send_otc_info_items(
    env: &Env,
    items_info: &Vec<OtcItemInfo>,
    to: &Addr,
) -> StdResult<Vec<CosmosMsg>> {
    let mut msgs: Vec<CosmosMsg> = vec![];
    for item_info in items_info {
        msgs.push(item_info.build_send_msg(env, &env.contract.address, to, None)?)
    }
    Ok(msgs)
}

pub fn assert_received_funds(items: &Vec<OtcItemInfo>, funds: Vec<Coin>) -> StdResult<Vec<Coin>> {
    let mut coins = rhaki_cw_plus::coin::vec_coins_to_hashmap(funds)?;

    for item in items {
        if let OtcItemInfo::Token { denom, amount } = &item {
            let available_amount = coins
                .get(denom)
                .ok_or(StdError::generic_err(format!("Coin not received {denom}")))?;

            if amount > available_amount {
                return Err(StdError::generic_err(format!(
                    "Amount received for {denom} is to low: expected: {amount}, received: {amount}"
                )));
            }

            coins.insert(denom.clone(), available_amount - amount);
        }
    }

    Ok(coins
        .into_iter()
        .map(|(denom, amount)| Coin::new(amount.u128(), denom))
        .collect())
}

pub fn after_action(
    deps: DepsMut,
    env: &Env,
    position: &mut OtcPosition,
) -> StdResult<Vec<Attribute>> {
    let position_pre = position.status.as_string_ref();

    position.try_close(env)?;

    let mut attributes: Vec<Attribute> = vec![];

    match position.status {
        OtcPositionStatus::Pending => {
            return Err(StdError::generic_err(
                "Position should be Executed or Vesting",
            ))
        }
        OtcPositionStatus::Vesting(_) | OtcPositionStatus::Executed(_) => {
            positions().save(deps.storage, position.id, position)?;
            let position_status_after = position.status.as_string_ref();

            if position_pre != position_status_after {
                attributes.push(attr("after_action", "position_status_change"));
                attributes.push(attr("pre_status", position_pre));
                attributes.push(attr("current_status", position_status_after));
            }
        }
    }

    Ok(attributes)
}
