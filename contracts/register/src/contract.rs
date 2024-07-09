use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use otcer_pkg::{
    register::{
        definitions::Config,
        msgs::{ExecuteMsg, InstantiateMsg, QueryMsg},
    },
    vesting_account::msgs::MigrateMsg,
};
use rhaki_cw_plus::traits::IntoAddr;

use crate::{execute::run_register_action, response::ContractResponse, state::CONFIG};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResponse {
    let config = Config {
        variable_provider: msg.variable_provider.into_addr(deps.api)?,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("variable_provider", config.variable_provider))
}

#[entry_point]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> ContractResponse {
    match msg {
        ExecuteMsg::RegisterAction(msg) => run_register_action(deps, info, msg),
    }
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[entry_point]
pub fn migrate(_deps: Deps, _env: Env, _msg: MigrateMsg) -> ContractResponse {
    unimplemented!()
}
