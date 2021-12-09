use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("Unauthorized")]
  Unauthorized {},

  #[error("Pair already exists")]
  PairExists {},

  #[error("Pair type already exists")]
  PairTypeExists {},

  #[error("Invalid tick space")]
  InvalidTickSpace {},

  #[error("Invalid fee rate")]
  InvalidFeeRate {},
}