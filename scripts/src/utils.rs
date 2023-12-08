use cosmos_grpc_client::{
    cosmos_sdk_proto::cosmos::tx::v1beta1::{GetTxRequest, GetTxResponse},
    GrpcClient,
};
use cosmwasm_std::{StdError, StdResult};

pub async fn search_tx(
    client: &mut GrpcClient,
    hash: String,
    _max_timeout: Option<u64>,
) -> StdResult<GetTxResponse> {
    loop {
        let res = client
            .clients
            .tx
            .get_tx(GetTxRequest { hash: hash.clone() })
            .await;

        if let Ok(response) = res {
            return Ok(response.into_inner());
        }
    }
}

pub fn get_code_id_from_init_response(response: GetTxResponse) -> StdResult<u64> {
    for event in response.tx_response.unwrap().events {
        // if event.r#type == "store_code".to_string() {
        for attribute in event.attributes {
            if attribute.key == *"code_id" {
                // clear code id
                // let a  = attribute.key.replace('"', "");
                return Ok(attribute.value.replace('"', "").parse().unwrap());
            }
        }
        // }
    }
    Err(StdError::generic_err("not found"))
}

pub fn get_address_from_init_response(response: GetTxResponse) -> StdResult<String> {
    for event in response.tx_response.unwrap().events {
        // if event.r#type == "instantiate".to_string() {
        for attribute in event.attributes {
            if attribute.key == *"_contract_address" {
                return Ok(attribute.value);
            }
        }
        // }
    }
    Err(StdError::generic_err("not found"))
}
