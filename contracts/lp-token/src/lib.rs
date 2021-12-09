mod error;
mod execute;
mod query;
mod state;

pub use crate::error::ContractError;
pub use wineswap::lp_token::{InstantiateMsg, ExecuteMsg, QueryMsg};
pub use crate::state::LpContract;

#[cfg(test)]
mod mock_querier;

#[cfg(test)]
mod testing;

#[cfg(not(feature = "library"))]
pub mod entry {
  use super::*;

  use cosmwasm_std::entry_point;
  use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

  #[entry_point]
  pub fn instantiate(
      deps: DepsMut,
      env: Env,
      info: MessageInfo,
      msg: InstantiateMsg,
  ) -> StdResult<Response> {
      let tract = LpContract::default();
      tract.instantiate(deps, env, info, msg)
  }

  #[entry_point]
  pub fn execute(
      deps: DepsMut,
      env: Env,
      info: MessageInfo,
      msg: ExecuteMsg,
  ) -> Result<Response, ContractError> {
    let tract = LpContract::default();
      tract.execute(deps, env, info, msg)
  }

  #[entry_point]
  pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let tract = LpContract::default();
      tract.query(deps, msg)
  }
}
