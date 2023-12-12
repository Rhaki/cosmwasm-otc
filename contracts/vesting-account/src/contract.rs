use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, StdResult};

use otcer_pkg::vesting_account::msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::response::ContractResponse;

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> ContractResponse {
    unimplemented!()
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> ContractResponse {
    unimplemented!()
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[entry_point]
pub fn migrate(_deps: Deps, _env: Env, _msg: MigrateMsg) -> StdResult<Binary> {
    unimplemented!()
}
