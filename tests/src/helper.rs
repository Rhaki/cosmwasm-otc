use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Empty, StdResult, Uint128};
use cw20::{BalanceResponse, Cw20Coin};
use cw721::OwnerOfResponse;
use cw_multi_test::{
    addons::{MockAddressGenerator, MockApiBech32},
    no_init, App, AppBuilder, AppResponse, BankKeeper, Executor, WasmKeeper,
};
use otcer_pkg::otcer::{
    definitions::{OtcItem, OtcItemInfo, OtcPosition},
    msgs::{CreateOtcMsg, ExecuteOtcMsg, OtcItemRegistration},
};
use rhaki_cw_plus::{
    math::{IntoDecimal, IntoUint},
    serde_value::{json, StdValue as Value},
    traits::IntoAddr,
};

use crate::{
    app_ext::{create_code, MergeCoin},
    cw721_value,
};

pub type AppResult = Result<AppResponse, anyhow::Error>;

pub type MyApp = App<BankKeeper, MockApiBech32>;

const BENCH32_PREFIX: &str = "terra";

#[cw_serde]
pub enum TokenType {
    Cw20,
    Native,
    Cw721,
}

#[derive(Debug)]
pub struct Def {
    pub addr_otc: Option<Addr>,
    pub code_id_cw20: Option<u64>,
    pub code_id_cw721: Option<u64>,
    pub fee_collector: Addr,
    pub owner: Addr,
    pub performance_fee: Decimal,
}

impl Def {
    pub fn new() -> Self {
        Self {
            addr_otc: None,
            code_id_cw20: None,
            code_id_cw721: None,
            fee_collector: generate_addr("fee_collector"),
            owner: generate_addr("owner"),
            performance_fee: "0.05".into_decimal(),
        }
    }
}

pub fn build_api() -> MockApiBech32 {
    MockApiBech32::new(BENCH32_PREFIX)
}

pub fn build_wasm_keeper() -> WasmKeeper<Empty, Empty> {
    WasmKeeper::default().with_address_generator(MockAddressGenerator)
}

pub fn generate_addr(name: &str) -> Addr {
    build_api().addr_make(name)
}

pub fn startup(def: &mut Def) -> MyApp {
    let mut app = AppBuilder::new()
        .with_api(build_api())
        .with_wasm(build_wasm_keeper())
        .build(no_init);

    let otcer_core_code_id = app.store_code(create_code(
        otcer_core::contract::instantiate,
        otcer_core::contract::execute,
        otcer_core::contract::query,
    ));

    let otcer_register_code_id = app.store_code(create_code(
        otcer_register::contract::instantiate,
        otcer_register::contract::execute,
        otcer_register::contract::query,
    ));

    let otcer_account_code_id = app.store_code(create_code(
        otcer_vesting_account::contract::instantiate,
        otcer_vesting_account::contract::execute,
        otcer_vesting_account::contract::query,
    ));

    let cw20_code_id = app.store_code(create_code(
        cw20_base::contract::instantiate,
        cw20_base::contract::execute,
        cw20_base::contract::query,
    ));

    let cw721_code_id = app.store_code(create_code(
        cw721_value::instantiate,
        cw721_value::execute,
        cw721_value::query,
    ));

    let variable_provider_code_id = app.store_code(create_code(
        variable_provider::contract::instantiate,
        variable_provider::contract::execute,
        variable_provider::contract::query,
    ));

    def.code_id_cw20 = Some(cw20_code_id);
    def.code_id_cw721 = Some(cw721_code_id);

    let otc_addr = app
        .instantiate_contract(
            otcer_core_code_id,
            def.owner.into_unchecked_addr(),
            &otcer_pkg::otcer::msgs::InstantiateMsg {
                owner: def.owner.to_string(),
                performance_fee: def.performance_fee.clone(),
                fee_collector: def.fee_collector.to_string(),
                code_id_variable_provider: variable_provider_code_id,
                code_id_vesting_account: otcer_account_code_id,
                code_id_register: otcer_register_code_id,
            },
            &[],
            "otc".to_string(),
            Some(def.owner.to_string()),
        )
        .unwrap();

    def.addr_otc = Some(otc_addr);

    app
}

fn native_funds_from_otc_item_registration(items: &[OtcItemRegistration]) -> Vec<Coin> {
    items
        .iter()
        .filter_map(|item| {
            if let OtcItemInfo::Token { denom, amount } = &item.item_info {
                Some(Coin::new(amount.u128(), denom))
            } else {
                None
            }
        })
        .collect()
}

fn native_funds_from_otc_item(items: &[OtcItem]) -> Vec<Coin> {
    items
        .iter()
        .filter_map(|item| {
            if let OtcItemInfo::Token { denom, amount } = &item.item_info {
                Some(Coin::new(amount.u128(), denom))
            } else {
                None
            }
        })
        .collect()
}

pub fn create_token(
    app: &mut MyApp,
    def: &mut Def,
    token_name: &str,
    token_type: TokenType,
    initial_balance: Vec<(&str, &str)>,
) -> Addr {
    match token_type {
        TokenType::Cw20 => app
            .instantiate_contract(
                def.code_id_cw20.unwrap(),
                def.owner.into_unchecked_addr(),
                &cw20_base::msg::InstantiateMsg {
                    name: token_name.to_string(),
                    symbol: token_name.to_string(),
                    decimals: 6,
                    initial_balances: initial_balance
                        .into_iter()
                        .map(|(to, amount)| Cw20Coin {
                            address: to.to_string(),
                            amount: amount.into_uint128(),
                        })
                        .collect(),
                    mint: Some(cw20::MinterResponse {
                        minter: def.owner.to_string(),
                        cap: None,
                    }),
                    marketing: None,
                },
                &[],
                token_name.to_string(),
                Some(def.owner.to_string()),
            )
            .unwrap(),
        TokenType::Cw721 => {
            let addr = app
                .instantiate_contract(
                    def.code_id_cw721.unwrap(),
                    def.owner.into_unchecked_addr(),
                    &cw721_base::msg::InstantiateMsg {
                        name: token_name.to_string(),
                        symbol: token_name.to_string(),
                        minter: def.owner.to_string(),
                    },
                    &[],
                    token_name.to_string(),
                    Some(def.owner.to_string()),
                )
                .unwrap();

            for (to, token_id) in initial_balance {
                mint_token(app, def, to, (addr.as_str(), token_type.clone()), token_id)
            }

            addr
        }
        TokenType::Native => todo!(),
    }
}

pub fn mint_token(
    app: &mut MyApp,
    def: &mut Def,
    to: impl Into<String>,
    token_info: (&str, TokenType),
    amount: &str,
) {
    match token_info.1 {
        TokenType::Cw20 => {
            app.execute_contract(
                def.owner.into_unchecked_addr(),
                token_info.0.into_unchecked_addr(),
                &cw20_base::msg::ExecuteMsg::Mint {
                    recipient: to.into(),
                    amount: amount.into_uint128(),
                },
                &[],
            )
            .unwrap();
        }
        TokenType::Native => {
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: to.into(),
                    amount: vec![Coin::new(amount.into_uint128().into(), token_info.0)],
                },
            ))
            .unwrap();
        }
        TokenType::Cw721 => {
            app.execute_contract(
                def.owner.into_unchecked_addr(),
                token_info.0.into_unchecked_addr(),
                &cw721_base::ExecuteMsg::Mint::<Value, Empty> {
                    token_id: amount.to_string(),
                    owner: to.into(),
                    token_uri: None,
                    extension: json!({}),
                },
                &[],
            )
            .unwrap();
        }
    }
}

pub fn increase_allowance(
    app: &mut MyApp,
    sender: impl Into<String>,
    to: impl Into<String>,
    addr: &Addr,
    token_type: TokenType,
    amount: &str,
) {
    match token_type {
        TokenType::Cw20 => {
            app.execute_contract(
                sender.into().into_unchecked_addr(),
                addr.clone(),
                &cw20::Cw20ExecuteMsg::IncreaseAllowance {
                    spender: to.into(),
                    amount: amount.into_uint128(),
                    expires: None,
                },
                &[],
            )
            .unwrap();
        }
        TokenType::Cw721 => {
            app.execute_contract(
                sender.into().into_unchecked_addr(),
                addr.clone(),
                &cw721_base::ExecuteMsg::Approve::<Value, Empty> {
                    spender: to.into(),
                    token_id: amount.to_string(),
                    expires: None,
                },
                &[],
            )
            .unwrap();
        }
        TokenType::Native => todo!(),
    }
}

// run

pub fn run_create_otc(
    app: &mut MyApp,
    def: &mut Def,
    creator: &str,
    executor: &str,
    offer: &[OtcItemRegistration],
    ask: &[OtcItemRegistration],
    mut extra_coin: Vec<Coin>,
) -> AppResult {
    let mut coins = native_funds_from_otc_item_registration(offer);

    coins.append(&mut extra_coin);

    let coins = coins.merge();

    app.execute_contract(
        creator.into_unchecked_addr(),
        def.addr_otc.clone().unwrap(),
        &otcer_pkg::otcer::msgs::ExecuteMsg::CreateOtc(CreateOtcMsg {
            executor: Some(executor.to_string()),
            offer: offer.to_vec(),
            ask: ask.to_vec(),
        }),
        &coins,
    )
}

pub fn run_execute_otc(
    app: &mut MyApp,
    def: &mut Def,
    sender: &str,
    id: u64,
    mut extra_coin: Vec<Coin>,
) -> AppResult {
    let position = qy_otc_position(app, def, id).unwrap();

    let mut coins = native_funds_from_otc_item(&position.ask);

    coins.append(&mut extra_coin);

    let coins = coins.merge();
    app.execute_contract(
        sender.into_unchecked_addr(),
        def.addr_otc.clone().unwrap(),
        &otcer_pkg::otcer::msgs::ExecuteMsg::ExecuteOtc(ExecuteOtcMsg { id }),
        &coins,
    )
}

// queries

pub fn qy_otc_position(app: &MyApp, def: &Def, id: u64) -> StdResult<OtcPosition> {
    app.wrap().query_wasm_smart(
        def.addr_otc.clone().unwrap(),
        &otcer_pkg::otcer::msgs::QueryMsg::Position { id },
    )
}



pub fn qy_balance_native(app: &MyApp, denom: &str, user: &str) -> Uint128 {
    app.wrap().query_balance(user, denom).unwrap().amount
}

pub fn qy_balance_cw20(app: &MyApp, addr: &Addr, user: &str) -> Uint128 {
    app.wrap()
        .query_wasm_smart::<BalanceResponse>(
            addr,
            &cw20::Cw20QueryMsg::Balance {
                address: user.to_string(),
            },
        )
        .unwrap()
        .balance
}

pub fn qy_balance_nft(app: &MyApp, addr: &Addr, token_id: &str, user: &str) -> bool {
    let owner = app
        .wrap()
        .query_wasm_smart::<OwnerOfResponse>(
            addr,
            &cw721::Cw721QueryMsg::OwnerOf {
                token_id: token_id.to_string(),
                include_expired: None,
            },
        )
        .unwrap()
        .owner;

    owner == *user
}
