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

  #[error("Upper tick must be greater than or equal to lower tick")]
  InvalidTickRange {},

  #[error("Zero liquidity error")]
  ZeroLiquidity {},

  #[error("Fail to update")]
  UpdateFail {},

  #[error("Can't swap")]
  CanNotSwap {},

  #[error("Max slippage assertion")]
  MaxSlippage,

  #[error("Asset mismatch")]
  AssetMismatch {},

  #[error("Tick range must be smaller or equal to 500")]
  TickRangeLimit {},

  #[error("You must put token_id or tick_indexes")]
  ProvideOptionError {},
}