use cosmwasm_std::{StdError, OverflowError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("{0}")]
  OverflowError(#[from] OverflowError),

  #[error("Unauthorized")]
  Unauthorized {},

  #[error("token_id already claimed")]
  Claimed {},

  #[error("Cannot set approval that is already expired")]
  Expired {},
}