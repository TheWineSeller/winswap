mod error;
mod execute;
mod response;
mod query;
mod state;

pub use crate::error::ContractError;
pub use wineswap::factory::{InstantiateMsg, ExecuteMsg, QueryMsg};
pub use crate::state::FactoryContract;

#[cfg(test)]
mod testing;

#[cfg(not(feature = "library"))]
pub mod entry {
  use super::*;

  use cosmwasm_std::entry_point;
  use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, Reply, StdResult};

  #[entry_point]
  pub fn instantiate(
      deps: DepsMut,
      env: Env,
      info: MessageInfo,
      msg: InstantiateMsg,
  ) -> StdResult<Response> {
      let tract = FactoryContract::default();
      tract.instantiate(deps, env, info, msg)
  }

  #[entry_point]
  pub fn reply(
    deps: DepsMut,
    env: Env,
    msg: Reply
  ) -> StdResult<Response> {
    let tract = FactoryContract::default();
    tract.reply(deps, env, msg)
  }

  #[entry_point]
  pub fn execute(
      deps: DepsMut,
      env: Env,
      info: MessageInfo,
      msg: ExecuteMsg,
  ) -> Result<Response, ContractError> {
    let tract = FactoryContract::default();
      tract.execute(deps, env, info, msg)
  }

  #[entry_point]
  pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let tract = FactoryContract::default();
      tract.query(deps, msg)
  }
}
