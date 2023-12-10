use cosmos_grpc_client::{
    cosmos_sdk_proto::cosmwasm::wasm::v1::{MsgExecuteContract, MsgInstantiateContract},
    cosmrs::tx::MessageExt,
    BroadcastMode, CoinType, GrpcClient, Wallet,
};
use cosmwasm_std::Empty;
use cw721::{ContractInfoResponse, NftInfoResponse, TokensResponse};
use otcer_scripts::utils::{get_address_from_init_response, search_tx};
use rhaki_cw_plus::{math::IntoDecimal, traits::IntoBinary};

const GRPC_FROM: &str = "https://terra-grpc.polkachu.com:11790";
const GRPC_TO: &str = "https://terra-testnet-grpc.polkachu.com:11790";
// TEST WALLET
const SEED_PHRASE: &str =
  "client duty genre image fancy image lake rescue doll thunder garage oppose source spare wise yellow moment theme bind alcohol motion tribe gas damage";
const ADDRESS_TO_CLONE: &str = "terra16ds898j530kn4nnlc7xlj6hcxzqpcxxk4mj8gkcl3vswksu6s3zszs8kp2"; // GP
const CODE_ID: u64 = 11809;
const CLONE_NUMBER: u64 = 20;

#[tokio::main]
async fn main() {
    let mut client_from = GrpcClient::new(GRPC_FROM).await.unwrap();
    let mut client_to = GrpcClient::new(GRPC_TO).await.unwrap();
    let mut wallet = Wallet::from_seed_phrase(
        &mut client_to,
        SEED_PHRASE,
        "terra",
        CoinType::Terra,
        0,
        "0.015".into_decimal(),
        "2".into_decimal(),
        "uluna",
    )
    .await
    .unwrap();

    let info: ContractInfoResponse = client_from
        .wasm_query_contract(ADDRESS_TO_CLONE, cw721::Cw721QueryMsg::ContractInfo {})
        .await
        .unwrap();

    let msg_init = MsgInstantiateContract {
        sender: wallet.account_address(),
        admin: wallet.account_address(),
        code_id: CODE_ID,
        label: info.name.clone(),
        msg: cw721_base::InstantiateMsg {
            name: info.name,
            symbol: info.symbol,
            minter: wallet.account_address(),
        }
        .into_binary()
        .unwrap()
        .to_vec(),
        funds: vec![],
    }
    .to_any()
    .unwrap();

    let response = wallet
        .broadcast_tx(
            &mut client_to,
            vec![msg_init],
            None,
            None,
            BroadcastMode::Sync,
        )
        .await
        .unwrap();

    println!("{response:#?}");

    let tx_info = search_tx(&mut client_to, response.tx_response.unwrap().txhash, None)
        .await
        .unwrap();

    let address = get_address_from_init_response(tx_info).unwrap();
    println!("address_nft: {address}");

    let tokens: TokensResponse = client_from
        .wasm_query_contract(
            ADDRESS_TO_CLONE,
            cw721::Cw721QueryMsg::AllTokens {
                start_after: None,
                limit: Some(CLONE_NUMBER as u32),
            },
        )
        .await
        .unwrap();

    let mut msgs = vec![];
    for token_id in tokens.tokens {
        println!("fetiching data {token_id}");
        let info: NftInfoResponse<cw721_metadata_onchain::Metadata> = client_from
            .wasm_query_contract(
                ADDRESS_TO_CLONE,
                cw721::Cw721QueryMsg::NftInfo {
                    token_id: token_id.clone(),
                },
            )
            .await
            .unwrap();

        let msg: cw721_base::ExecuteMsg<cw721_metadata_onchain::Metadata, Empty> =
            cw721_base::ExecuteMsg::Mint {
                token_id,
                owner: wallet.account_address(),
                token_uri: info.token_uri,
                extension: info.extension,
            };

        let msg = MsgExecuteContract {
            sender: wallet.account_address(),
            contract: address.clone(),
            msg: msg.into_binary().unwrap().to_vec(),
            funds: vec![],
        }
        .to_any()
        .unwrap();

        msgs.push(msg);
    }

    let res = wallet
        .broadcast_tx(&mut client_to, msgs, None, None, BroadcastMode::Sync)
        .await
        .unwrap();

    print!("{res:#?}")
}
