use cosmwasm_std::{ConversionOverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
}

impl From<ConversionOverflowError> for ContractError {
    fn from(value: ConversionOverflowError) -> Self {
        ContractError::Std(StdError::generic_err(value.value))
    }
}