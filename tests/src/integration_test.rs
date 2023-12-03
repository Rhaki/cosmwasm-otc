use cosmwasm_otc_pkg::otc::{definitions::OtcItemInfo, msgs::OtcItemRegistration};

use crate::helper::{
    create_token, increase_allowance, mint_token, qy_balance_cw20, qy_balance_native,
    qy_balance_nft, run_create_otc, startup, Def, TokenType,
};

#[test]
#[rustfmt::skip]
pub fn test() {
    let mut def = Def::new();

    let mut app = startup(&mut def);

    let creator = "creator";
    let dealer = "dealer";

    let fee = def.get_native_fee();
   
    // Create tokens

    let offer_nft_id = "1";
    let offer_cw20_amount= 100_u128;
    let offer_native_amount= 150_u128;

    let offer_nft_addr = create_token(&mut app, &mut def, "NftOffer", TokenType::Cw721, vec![(creator, offer_nft_id)]);
    let offer_cw20_addr = create_token(&mut app, &mut def, "TokenOffer", TokenType::Cw20, vec![(creator, &offer_cw20_amount.to_string())]);
    let offer_native_denom = "luna";
    mint_token(&mut app, &mut def, creator, (offer_native_denom, TokenType::Native), &offer_native_amount.to_string());

    let ask_nft_id = "2";
    let ask_cw20_amount= 200_u128;
    let ask_native_amount= 250_u128;

    let ask_nft_addr = create_token(&mut app, &mut def, "NftOffer", TokenType::Cw721, vec![(dealer, ask_nft_id)]);
    let ask_cw20_addr = create_token(&mut app, &mut def, "TokenOffer", TokenType::Cw20, vec![(dealer, &ask_cw20_amount.to_string())]);
    let ask_native_denom = "btc";
    mint_token(&mut app, &mut def, dealer, (ask_native_denom, TokenType::Native), &ask_native_amount.to_string());

    // Increase allowance

    increase_allowance(&mut app, creator, &def.addr_otc.clone().unwrap().to_string(), &offer_nft_addr, TokenType::Cw721, offer_nft_id);
    increase_allowance(&mut app, creator, &def.addr_otc.clone().unwrap().to_string(), &offer_cw20_addr, TokenType::Cw20, &offer_cw20_amount.to_string());

    // Create otc

    let offer_items = vec![
        OtcItemRegistration { info: OtcItemInfo::Token { denom: offer_native_denom.to_string(), amount: offer_native_amount.into() }, vesting: None },
        OtcItemRegistration { info: OtcItemInfo::Cw20 { contract: offer_cw20_addr.clone(), amount: offer_cw20_amount.into() }, vesting: None },
        OtcItemRegistration { info: OtcItemInfo::Cw721 { contract: offer_nft_addr.clone(), token_id: offer_nft_id.to_string() }, vesting: None }
    ];

    let ask_items = vec![
        OtcItemRegistration { info: OtcItemInfo::Token { denom: ask_native_denom.to_string(), amount: ask_native_amount.into() }, vesting: None },
        OtcItemRegistration { info: OtcItemInfo::Cw20 { contract: ask_cw20_addr.clone(), amount: ask_cw20_amount.into() }, vesting: None },
        OtcItemRegistration { info: OtcItemInfo::Cw721 { contract: ask_nft_addr, token_id: ask_nft_id.to_string() }, vesting: None }
    ];

    // fails for missing fee

    run_create_otc(&mut app, &mut def, creator, dealer, &offer_items, &ask_items, vec![]).unwrap_err();

    mint_token(&mut app, &mut def, creator, (&fee[0].denom, TokenType::Native), &fee[0].amount.to_string());
    run_create_otc(&mut app, &mut def, creator, dealer, &offer_items, &ask_items, fee).unwrap();

    // assert position

    assert_eq!(offer_cw20_amount, qy_balance_cw20(&app, &offer_cw20_addr, &def.addr_otc.clone().unwrap().to_string()).u128());
    assert_eq!(offer_native_amount, qy_balance_native(&app, &offer_native_denom, &def.addr_otc.clone().unwrap().to_string()).u128());
    assert_eq!(true, qy_balance_nft(&app, &offer_nft_addr, offer_nft_id, &def.addr_otc.clone().unwrap().to_string()));
  
}
