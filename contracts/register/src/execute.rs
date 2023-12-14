use cosmwasm_std::{DepsMut, MessageInfo};
use otcer_pkg::register::msgs::RegisterActionMsg;

use crate::response::ContractResponse;

pub fn run_register_action(
    deps: DepsMut,
    info: MessageInfo,
    msg: RegisterActionMsg,
) -> ContractResponse {
    msg.asset
        .validate_permissionless_registration(deps.as_ref(), info.sender)?;

    todo!()
}
