use cosmwasm_std::Decimal;
use otcer_pkg::otcer::{definitions::{OtcItemInfo, OtcItem}, msgs::OtcItemRegistration};
use rhaki_cw_plus::math::IntoUint;

use crate::helper::{
    create_token, generate_addr, increase_allowance, mint_token, qy_balance_cw20,
    qy_balance_native, qy_balance_nft, qy_otc_position, run_create_otc, run_execute_otc, startup,
    Def, TokenType,
};

#[test]
#[rustfmt::skip]
pub fn test() {
    let mut def = Def::new();

    let mut app = startup(&mut def);

    let creator =  generate_addr("creator").to_string();
    let executor =  generate_addr("executor").to_string();

    let fee = def.performance_fee;
   
    // Create tokens

    let offer_nft_id = "1";
    let offer_cw20_amount= 100_u128;
    let offer_native_amount= 150_u128;

    let offer_nft_addr = create_token(&mut app, &mut def, "NftOffer", TokenType::Cw721, vec![(&creator, offer_nft_id)]);
    let offer_cw20_addr = create_token(&mut app, &mut def, "TokenOffer", TokenType::Cw20, vec![(&creator, &offer_cw20_amount.to_string())]);
    let offer_native_denom = "luna";
    mint_token(&mut app, &mut def, &creator, (offer_native_denom, TokenType::Native), &offer_native_amount.to_string());

    let ask_nft_id = "2";
    let ask_cw20_amount= 200_u128;
    let ask_native_amount= 250_u128;

    let ask_nft_addr = create_token(&mut app, &mut def, "NftOffer", TokenType::Cw721, vec![(&executor, ask_nft_id)]);
    let ask_cw20_addr = create_token(&mut app, &mut def, "TokenOffer", TokenType::Cw20, vec![(&executor, &ask_cw20_amount.to_string())]);
    let ask_native_denom = "btc";
    mint_token(&mut app, &mut def, &executor, (ask_native_denom, TokenType::Native), &ask_native_amount.to_string());

    // Increase allowance

    increase_allowance(&mut app, &creator, def.addr_otc.clone().unwrap().as_ref(), &offer_nft_addr, TokenType::Cw721, offer_nft_id);
    increase_allowance(&mut app, &creator, def.addr_otc.clone().unwrap().as_ref(), &offer_cw20_addr, TokenType::Cw20, &offer_cw20_amount.to_string());

    // Create otc

    let offer_items = vec![
        OtcItemRegistration { item_info: OtcItemInfo::Token { denom: offer_native_denom.to_string(), amount: offer_native_amount.into() }, vesting: None },
        OtcItemRegistration { item_info: OtcItemInfo::Cw20 { contract: offer_cw20_addr.clone(), amount: offer_cw20_amount.into() }, vesting: None },
        OtcItemRegistration { item_info: OtcItemInfo::Cw721 { contract: offer_nft_addr.clone(), token_id: offer_nft_id.to_string() }, vesting: None }
    ];

    let ask_items = vec![
        OtcItemRegistration { item_info: OtcItemInfo::Token { denom: ask_native_denom.to_string(), amount: ask_native_amount.into() }, vesting: None },
        OtcItemRegistration { item_info: OtcItemInfo::Cw20 { contract: ask_cw20_addr.clone(), amount: ask_cw20_amount.into() }, vesting: None },
        OtcItemRegistration { item_info: OtcItemInfo::Cw721 { contract: ask_nft_addr.clone(), token_id: ask_nft_id.to_string() }, vesting: None }
    ];

    // fails for missing fee
    run_create_otc(&mut app, &mut def, &creator, &executor, &offer_items, &ask_items, vec![]).unwrap();

    let new_offer_native_amount = offer_native_amount.into_uint128() - offer_native_amount.into_uint128() * fee;
    let new_offer_cw20_amount = offer_cw20_amount.into_uint128() - offer_cw20_amount.into_uint128() * fee;

    // assert position
    assert_eq!(new_offer_cw20_amount, qy_balance_cw20(&app, &offer_cw20_addr, def.addr_otc.clone().unwrap().as_ref()));
    assert_eq!(new_offer_native_amount, qy_balance_native(&app, offer_native_denom, def.addr_otc.clone().unwrap().as_ref()));

    assert_eq!(offer_cw20_amount.into_uint128() * fee, qy_balance_cw20(&app, &offer_cw20_addr, def.fee_collector.as_str()));
    assert_eq!(offer_native_amount.into_uint128() * fee, qy_balance_native(&app, offer_native_denom, def.fee_collector.as_str()));

    assert!(qy_balance_nft(&app, &offer_nft_addr, offer_nft_id, def.addr_otc.clone().unwrap().as_ref()));
    assert_eq!(qy_otc_position(&app, &def, 1).unwrap().offer,vec![
        OtcItem { item_info: OtcItemInfo::Token { denom: offer_native_denom.to_string(), amount: new_offer_native_amount }, vesting_info: None },
        OtcItem { item_info: OtcItemInfo::Cw20 { contract: offer_cw20_addr.clone(), amount: new_offer_cw20_amount }, vesting_info: None },
        OtcItem { item_info: OtcItemInfo::Cw721 { contract: offer_nft_addr.clone(), token_id: offer_nft_id.to_string() }, vesting_info: None }
    ]);

    // close position
    increase_allowance(&mut app, &executor, def.addr_otc.clone().unwrap().as_ref(), &ask_nft_addr, TokenType::Cw721, ask_nft_id);
    increase_allowance(&mut app, &executor, def.addr_otc.clone().unwrap().as_ref(), &ask_cw20_addr, TokenType::Cw20, &ask_cw20_amount.to_string());

    run_execute_otc(&mut app, &mut def, &executor, 1, vec![]).unwrap();

    let new_ask_native_amount = ask_native_amount.into_uint128() - ask_native_amount.into_uint128() * fee;
    let new_ask_cw20_amount = ask_cw20_amount.into_uint128() - ask_cw20_amount.into_uint128() * fee;

    // assert result
    assert_eq!(new_offer_cw20_amount, qy_balance_cw20(&app, &offer_cw20_addr, &executor));
    assert_eq!(new_offer_native_amount, qy_balance_native(&app, offer_native_denom, &executor));
    assert!(qy_balance_nft(&app, &offer_nft_addr, offer_nft_id, &executor));

    assert_eq!(new_ask_cw20_amount, qy_balance_cw20(&app, &ask_cw20_addr, &creator));
    assert_eq!(new_ask_native_amount, qy_balance_native(&app, ask_native_denom, &creator));
    assert!(qy_balance_nft(&app, &ask_nft_addr, ask_nft_id, &creator));

    assert_eq!(ask_cw20_amount.into_uint128() * fee, qy_balance_cw20(&app, &ask_cw20_addr, def.fee_collector.as_str()));
    assert_eq!(ask_native_amount.into_uint128() * fee, qy_balance_native(&app, ask_native_denom, def.fee_collector.as_str()));
    
    assert_eq!("executed", qy_otc_position(&app, &def, 1).unwrap().status.as_string_ref());

}
