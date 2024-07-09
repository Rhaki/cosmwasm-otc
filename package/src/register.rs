pub mod msgs {
    use cosmwasm_schema::{cw_serde, QueryResponses};
    use rhaki_cw_plus::{cw_serde_value, serde_value::Value};

    use super::definitions::{InputVariable, RegisterAssetType};

    #[cw_serde]
    pub struct InstantiateMsg {
        pub owner: String,
        pub variable_provider: String,
    }

    #[cw_serde_value]
    pub enum ExecuteMsg {
        RegisterAction(RegisterActionMsg),
    }

    #[cw_serde]
    #[derive(QueryResponses)]
    pub enum QueryMsg {}

    #[cw_serde]
    pub struct MigrateMsg {}

    #[cw_serde_value]
    pub struct RegisterActionMsg {
        pub asset: RegisterAssetType,
        pub name: String,
        pub input_variable: Vec<InputVariable>,
        pub messages: Vec<Value>,
    }
}

pub mod definitions {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Addr, Deps, StdError, StdResult};
    use cw20::MinterResponse;
    use rhaki_cw_plus::traits::IntoAddr;

    #[cw_serde]
    pub struct Config {
        pub variable_provider: Addr,
    }

    #[cw_serde]
    pub enum RegisterAssetType {
        Native(String),
        Cw20(String),
        Cw721(String),
    }

    impl RegisterAssetType {
        pub fn as_cw20(&self) -> StdResult<String> {
            match self {
                RegisterAssetType::Cw20(addr) => Ok(addr.clone()),
                _ => Err(StdError::generic_err("RegisterAssetType is Not cw20")),
            }
        }

        pub fn validate_permissionless_registration(
            &self,
            deps: Deps,
            sender: Addr,
        ) -> StdResult<()> {
            match self {
                RegisterAssetType::Cw20(raw_cw20_addr) => {
                    RegisterAssetType::validate_cw20_registration(
                        deps,
                        sender,
                        raw_cw20_addr.clone(),
                    )
                }
                _ => Err(StdError::generic_err(format!(
                    "validate_permissionless_registration not handled for {self:#?}"
                ))),
            }
        }

        fn validate_cw20_registration(
            deps: Deps,
            sender: Addr,
            raw_cw20_addr: String,
        ) -> StdResult<()> {
            let cw20_addr = raw_cw20_addr.into_addr(deps.api)?;
            let admin = deps
                .querier
                .query_wasm_contract_info(cw20_addr.clone())?
                .admin;
            let minter = deps
                .querier
                .query_wasm_smart::<MinterResponse>(cw20_addr, &cw20::Cw20QueryMsg::Minter {})?
                .minter;

            if !(minter == sender || admin.unwrap_or("".to_string()) == sender) {
                return Err(StdError::generic_err("Unauthorized"));
            }

            Ok(())
        }
    }

    #[cw_serde]
    pub struct InputVariable {
        pub name: String,
        pub variable_type: InputVariableType,
    }

    #[cw_serde]
    pub enum InputVariableType {
        String,
        Decimal,
        U64,
        Uint,
    }
}
