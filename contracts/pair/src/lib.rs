mod error;
mod execute;
mod query;
mod state;
mod response;

pub use wineswap::pair::{InstantiateMsg, ExecuteMsg, QueryMsg};
pub use crate::error::ContractError;
pub use crate::state::PairContract;

#[cfg(test)]
mod testing;

#[cfg(test)]
mod mock_querier;


#[cfg(not(feature = "library"))]
pub mod entry {
  use super::*;

  use cosmwasm_std::entry_point;
  use cosmwasm_std::{Binary, Deps, DepsMut, Env, Reply, MessageInfo, Response, StdResult};

  #[entry_point]
  pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
  ) -> StdResult<Response> {
    let tract = PairContract::default();
    tract.instantiate(deps, env, info, msg)
  }

  #[entry_point]
  pub fn reply(
    deps: DepsMut,
    env: Env,
    msg: Reply
  ) -> StdResult<Response> {
    let tract = PairContract::default();
    tract.reply(deps, env, msg)
  }

  #[entry_point]
  pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
  ) -> Result<Response, ContractError> {
    let tract = PairContract::default();
    tract.execute(deps, env, info, msg)
  }

  #[entry_point]
  pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let tract = PairContract::default();
      tract.query(deps, msg)
  }
}
