use cosmwasm_std::{Decimal, Response, StdError};
use thiserror::Error;

pub type ContractResponse = Result<Response, ContractError>;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Invalid performance fee: {fee}")]
    InvalidPerformanceFee { fee: Decimal },

    #[error("Wrong coins sent during creating or executing otc")]
    ExtraCoinReceived
}
