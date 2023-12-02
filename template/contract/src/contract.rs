use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, StdResult};

use $pkg$::$contract$::msgs::{ExecuteMsg, InstantiateMsg, QueryMsg};

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
