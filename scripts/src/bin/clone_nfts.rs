use cosmos_grpc_client::GrpcClient;

const RPC_FROM: &str = "https://terra-rpc.polkachu.com";
const RPC_TO: &str = "https://injective-testnet-grpc.polkachu.com:14390";
const SEED_PHRASE: &str =
  "client duty genre image fancy image lake rescue doll thunder garage oppose source spare wise yellow moment theme bind alcohol motion tribe gas damage";
const ADDRESS_TO_CLONE: &str = "terra16ds898j530kn4nnlc7xlj6hcxzqpcxxk4mj8gkcl3vswksu6s3zszs8kp2"; // GP
const CODE_ID: u64 = 11809;
const CLONE_NUMBER: u64 = 20;

#[tokio::main]
async fn main() {
    let client_from = GrpcClient::new(RPC_FROM).await;
    let client_to = GrpcClient::new(RPC_FROM).await;
}
