use cosmwasm_std::{
    entry_point, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};

use otcer_pkg::otcer::{
    definitions::Config,
    msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};
use rhaki_cw_plus::{
    traits::{IntoAddr, IntoBinary, IntoBinaryResult},
    wasm::WasmMsgBuilder,
};
use variable_provider_pkg::{definitions::Variable, msgs::RegisterVariableMsg};

use crate::{
    execute::{run_cancel_otc, run_claim_otc, run_create_otc, run_execute_otc},
    query::{qy_position, qy_positions},
    response::ContractResponse,
    state::CONFIG,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResponse {
    // Asserts
    if msg.performance_fee.is_zero() || msg.performance_fee > Decimal::one() {
        return Err(crate::response::ContractError::InvalidPerformanceFee {
            fee: msg.performance_fee,
        });
    }

    // Init variable provider
    let (msg_init_vp, variable_provider_addr) = rhaki_cw_plus::wasm::build_instantiate_2(
        deps.as_ref(),
        &env.contract.address,
        "variable_provider".to_string().into_binary()?,
        Some(msg.owner.to_string()),
        msg.code_id_variable_provider,
        variable_provider_pkg::msgs::InstantiateMsg {
            owners: vec![env.contract.address.to_string(), msg.owner.clone()],
        },
        vec![],
        "Otcer variable provider".to_string(),
    )?;

    // Init register
    let (msg_init_register, register_addr) = rhaki_cw_plus::wasm::build_instantiate_2(
        deps.as_ref(),
        &env.contract.address,
        "register".to_string().into_binary()?,
        Some(msg.owner.to_string()),
        msg.code_id_register,
        otcer_pkg::register::msgs::InstantiateMsg {
            owner: msg.owner.clone(),
            variable_provider: variable_provider_addr.to_string(),
        },
        vec![],
        "Otcer register".to_string(),
    )?;

    // Register fee_collector
    let msg_register_fee_collector = WasmMsg::build_execute(
        &variable_provider_addr,
        variable_provider_pkg::msgs::ExecuteMsg::RegisterVariable(RegisterVariableMsg {
            key: otcer_pkg::variable_provider::KEY_FEE_COLLECTOR_ADDR.to_string(),
            value: Variable::Addr(msg.fee_collector.clone().into_addr(deps.api)?),
        }),
        vec![],
    )?;

    // Register performance fee
    let msg_register_fees = WasmMsg::build_execute(
        &variable_provider_addr,
        variable_provider_pkg::msgs::ExecuteMsg::RegisterVariable(RegisterVariableMsg {
            key: otcer_pkg::variable_provider::KEY_PERFORMANCE_FEE.to_string(),
            value: Variable::Decimal(msg.performance_fee),
        }),
        vec![],
    )?;

    // Register register
    let msg_register_register = WasmMsg::build_execute(
        &variable_provider_addr,
        variable_provider_pkg::msgs::ExecuteMsg::RegisterVariable(RegisterVariableMsg {
            key: otcer_pkg::variable_provider::KEY_REGISTER_ADDR.to_string(),
            value: Variable::Addr(register_addr.clone()),
        }),
        vec![],
    )?;

    let config = Config::new(
        msg.owner.clone().into_addr(deps.api)?,
        variable_provider_addr.clone(),
    );

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", msg.owner)
        .add_attribute("variable_provider", variable_provider_addr)
        .add_attribute("register", register_addr)
        .add_attribute("fee_collector", msg.fee_collector)
        .add_attribute("performance_fee", msg.performance_fee.to_string())
        .add_message(msg_init_vp)
        .add_message(msg_init_register)
        .add_message(msg_register_fee_collector)
        .add_message(msg_register_fees)
        .add_message(msg_register_register))
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
        QueryMsg::Position { id } => qy_position(deps, id).into_binary(),
        QueryMsg::Positions {
            limit,
            start_after,
            filters,
            order,
        } => qy_positions(deps, start_after, limit, filters, order).into_binary(),
    }
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResponse {
    Ok(Response::default())
}
