pub mod msgs {
    use cosmwasm_schema::{cw_serde, QueryResponses};

    use super::definitions::{OtcItem, OtcPosition};

    #[cw_serde]
    pub struct InstantiateMsg {
        pub owner: String,
        pub fee: Vec<OtcItem>,
        pub fee_collector: String,
    }

    #[cw_serde]
    pub enum ExecuteMsg {
        CreateOtc(CreateOtcMsg),
        ExecuteOtc(ExecuteOtcMsg),
        CancelOtc(CancelOtcMsg),
    }

    #[cw_serde]
    pub struct CreateOtcMsg {
        pub dealer: Option<String>,
        pub offer: Vec<OtcItem>,
        pub ask: Vec<OtcItem>,
        pub expiration_time: Option<u64>,
    }

    #[cw_serde]
    pub struct ExecuteOtcMsg {
        pub id: u64,
    }

    #[cw_serde]
    pub struct CancelOtcMsg {
        pub id: u64,
    }

    #[cw_serde]
    #[derive(QueryResponses)]
    pub enum QueryMsg {
        #[returns(OtcPosition)]
        ActivePosition { id: u64 },
        #[returns(OtcPosition)]
        ExecutedPosition { id: u64 },
        #[returns(Vec<OtcPosition>)]
        ActivePositions {
            limit: Option<u32>,
            start_after: Option<u64>,
        },
        #[returns(Vec<OtcPosition>)]
        ExecutedPositions {
            limit: Option<u32>,
            start_after: Option<u64>,
        },
        #[returns(Vec<OtcPosition>)]
        ActivePositionsByOwner {
            owner: String,
            limit: Option<u32>,
            start_after: Option<u64>,
        },
        #[returns(Vec<OtcPosition>)]
        ActrivePositionByDealer {
            dealer: String,
            limit: Option<u32>,
            start_after: Option<u64>,
        },
        #[returns(Vec<OtcPosition>)]
        ExecutedPositionsByOwner {
            owner: String,
            limit: Option<u32>,
            start_after: Option<u64>,
        },
        #[returns(Vec<OtcPosition>)]
        ExecutedPositionBtDealer {
            dealer: String,
            limit: Option<u32>,
            start_after: Option<u64>,
        },
    }

    #[cw_serde]
    pub struct MigrateMsg {}
}

pub mod definitions {
    use std::collections::HashMap;

    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Addr, Coin, CosmosMsg, Deps, StdError, StdResult, Uint128, WasmMsg};
    use rhaki_cw_plus::{traits::IntoAddr, wasm::WasmMsgBuilder};

    use super::msgs::CreateOtcMsg;

    #[cw_serde]
    pub struct Config {
        pub owner: Addr,
        pub counter_otc: u64,
        pub fee: Vec<OtcItem>,
        pub fee_collector: Addr,
    }

    impl Config {
        pub fn new(
            deps: Deps,
            owner: Addr,
            fee: Vec<OtcItem>,
            fee_collector: Addr,
        ) -> StdResult<Config> {
            for i in &fee {
                i.validate(deps)?;
            }

            Ok(Config {
                owner,
                counter_otc: 0,
                fee,
                fee_collector,
            })
        }
    }

    #[cw_serde]
    pub enum OtcItem {
        Token { denom: String, amount: Uint128 },
        Cw20 { contract: Addr, amount: Uint128 },
        Cw721 { contract: Addr, token_id: String },
    }

    impl OtcItem {
        pub fn validate(&self, deps: Deps) -> StdResult<()> {
            match self {
                OtcItem::Token { .. } => Ok(()),
                OtcItem::Cw20 { contract, .. } => {
                    contract.to_string().into_addr(deps.api).map(|_| ())
                }
                OtcItem::Cw721 { contract, .. } => {
                    contract.to_string().into_addr(deps.api).map(|_| ())
                }
            }
        }
    }

    #[cw_serde]
    pub struct OtcPosition {
        pub id: u64,
        pub owner: Addr,
        pub dealer: Option<Addr>,
        pub offer: Vec<OtcItem>,
        pub ask: Vec<OtcItem>,
        pub expiration_time: Option<u64>,
    }

    impl OtcPosition {
        pub fn validate(&self, deps: Deps) -> StdResult<()> {
            if let Some(dealer) = &self.dealer {
                dealer.to_string().into_addr(deps.api)?;
            }

            for item in self.offer.iter().chain(self.ask.iter()) {
                item.validate(deps)?;
            }

            Ok(())
        }
        pub fn from_create_otc_msg(
            deps: Deps,
            msg: CreateOtcMsg,
            id: u64,
            owner: Addr,
        ) -> StdResult<OtcPosition> {
            Ok(OtcPosition {
                id,
                owner,
                dealer: msg.dealer.map(|val| val.into_addr(deps.api)).transpose()?,
                offer: msg.offer,
                ask: msg.ask,
                expiration_time: msg.expiration_time,
            })
        }
    }

    pub trait OtcItemsChecker {
        fn gather_items(
            &self,
            to: Addr,
            sender: Addr,
            funds: Option<Vec<Coin>>,
        ) -> StdResult<(Vec<CosmosMsg>, Vec<Coin>)>;
    }

    impl OtcItemsChecker for Vec<OtcItem> {
        fn gather_items(
            &self,
            to: Addr,
            sender: Addr,
            funds: Option<Vec<Coin>>,
        ) -> StdResult<(Vec<CosmosMsg>, Vec<Coin>)> {
            let mut coins = if let Some(funds) = &funds {
                rhaki_cw_plus::coin::vec_coins_to_hashmap(funds.clone())?
            } else {
                HashMap::default()
            };

            let mut msgs: Vec<CosmosMsg> = vec![];
            for item in self {
                match item {
                    OtcItem::Token { denom, amount } => {
                        if funds.is_some() {
                            let available_amount = coins.get(denom).ok_or(
                                StdError::generic_err(format!("Coin not received {denom}")),
                            )?;

                            if amount > available_amount {
                                return Err(StdError::generic_err(format!("Amount received for {denom} is to low: expected: {amount}, received: {amount}")));
                            }

                            coins.insert(denom.clone(), available_amount - amount);
                        }
                    }
                    OtcItem::Cw20 { contract, amount } => msgs.push(
                        WasmMsg::build_execute(
                            contract,
                            cw20::Cw20ExecuteMsg::TransferFrom {
                                owner: sender.to_string(),
                                recipient: to.to_string(),
                                amount: amount.to_owned(),
                            },
                            vec![],
                        )?
                        .into(),
                    ),
                    OtcItem::Cw721 { contract, token_id } => msgs.push(
                        WasmMsg::build_execute(
                            contract,
                            cw721::Cw721ExecuteMsg::TransferNft {
                                recipient: to.to_string(),
                                token_id: token_id.to_owned(),
                            },
                            vec![],
                        )?
                        .into(),
                    ),
                }
            }

            Ok((
                msgs,
                coins
                    .into_iter()
                    .map(|(denom, amount)| Coin::new(amount.u128(), denom))
                    .collect(),
            ))
        }
    }
}
