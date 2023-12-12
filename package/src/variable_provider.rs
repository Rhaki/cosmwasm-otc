use cosmwasm_std::{Addr, Decimal, Deps, StdResult};

pub const KEY_FEE_COLLECTOR_ADDR: &str = "fee_collector_addr";
pub const KEY_PERFORMANCE_FEE: &str = "performance_fee";
pub const KEY_REGISTER_ADDR: &str = "register_addr";

pub fn vp_get_fee_and_collector(deps: Deps, vp_addr: &Addr) -> StdResult<(Decimal, Addr)> {
    let result = variable_provider_pkg::helper::variable_provider_get_variables(
        deps,
        vec![KEY_FEE_COLLECTOR_ADDR, KEY_PERFORMANCE_FEE],
        vp_addr,
    )?;

    Ok((
        result.get(KEY_PERFORMANCE_FEE).unwrap().unwrap_decimal()?,
        result.get(KEY_FEE_COLLECTOR_ADDR).unwrap().unwrap_addr()?,
    ))
}
