use cosmwasm_std::{to_binary, Decimal, WasmQuery, QueryRequest, Binary, Deps, QuerierWrapper, Order, StdError, StdResult};

use cw_storage_plus::Bound;

pub use wineswap::lp_token::{
  QueryMsg, OwnerOfResponse, ConfigResponse, LiquidityInfoResponse, TokensResponse, MinterResponse, RewardResponse
};
pub use wineswap::pair::{TickInfosResponse, PairInfoResponse, QueryMsg as PairQueryMsg};
pub use wineswap::asset::Asset;
use crate::state::{LpContract, FeeInfo};

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

impl<'a> LpContract<'a> {
  fn owner_of(&self, deps: Deps, token_id: String) -> StdResult<OwnerOfResponse> {
    let token = self.tokens.load(deps.storage, &token_id)?;
    Ok(OwnerOfResponse {
      owner: token.owner.to_string()
    })
  }

  fn config(&self, deps: Deps) -> StdResult<ConfigResponse> {
    self.config.load(deps.storage)
  }

  fn liquidity_info(&self, deps: Deps, token_id: String) -> StdResult<LiquidityInfoResponse> {
    let token = self.tokens.load(deps.storage, &token_id)?;
    Ok(LiquidityInfoResponse {
      owner: token.owner,
      approvals: token.approvals,
      liquidity: token.liquidity,
      upper_tick_index: token.upper_tick_index,
      lower_tick_index: token.lower_tick_index,
    })
  }

  fn tokens(
    &self,
    deps: Deps,
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
  ) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let owner_addr = deps.api.addr_validate(&owner)?;
    let pks: Vec<_> = self
      .tokens
      .idx
      .owner
      .prefix(owner_addr)
      .keys(deps.storage, start, None, Order::Ascending)
      .take(limit)
      .collect();

    let res: Result<Vec<_>, _> = pks.iter().map(|v| String::from_utf8(v.to_vec())).collect();
    let tokens = res.map_err(StdError::invalid_utf8)?;
    Ok(TokensResponse { tokens })
  }

  fn all_tokens(
    &self,
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
  ) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let tokens: StdResult<Vec<String>> = self
      .tokens
      .range(deps.storage, start, None, Order::Ascending)
      .take(limit)
      .map(|item| item.map(|(k, _)| String::from_utf8_lossy(&k).to_string()))
      .collect();

    Ok(TokensResponse { tokens: tokens? })
  }

  fn minter(&self, deps: Deps) -> StdResult<MinterResponse> {
    let config = self.config.load(deps.storage)?;
    let minter_addr = config.minter;
    Ok(MinterResponse {
      minter: minter_addr.to_string(),
    })
  }

  pub fn reward(&self, deps: Deps, token_id: String) -> StdResult<RewardResponse> {
    let token = self.tokens.load(deps.storage, &token_id)?;
    let config = self.config.load(deps.storage)?;
    let pair_config: PairInfoResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
      contract_addr: config.minter.to_string(),
      msg: to_binary(&PairQueryMsg::PairInfo{})?,
    }))?;
    let asset_infos = pair_config.asset_infos;
    let new_infos: Vec<FeeInfo> = self.get_fee_infos(deps.querier, config.minter.to_string(), token.upper_tick_index, token.lower_tick_index)?;
    
    let mut reward_per_liquidity_0 = Decimal::zero();
    let mut reward_per_liquidity_1 = Decimal::zero();
    
    // sum (global state - lp token state) of each growth
    for i in 0..new_infos.len() {
      reward_per_liquidity_0 = reward_per_liquidity_0 + new_infos[i].last_fee_growth_0 - token.last_updated_fee_infos[i].last_fee_growth_0;
      reward_per_liquidity_1 = reward_per_liquidity_1 + new_infos[i].last_fee_growth_1 - token.last_updated_fee_infos[i].last_fee_growth_1;
    }

    let rewards = [
      Asset{
        info: asset_infos[0].clone(),
        amount: reward_per_liquidity_0 * token.liquidity
      },
      Asset{
        info: asset_infos[1].clone(),
        amount: reward_per_liquidity_1 * token.liquidity
      },
    ];
    
    Ok(RewardResponse{
      rewards
    })
  }

  pub fn get_fee_infos(&self, querier: QuerierWrapper, pair_contract: String, upper_tick_index: i32, lower_tick_index:i32) -> StdResult<Vec<FeeInfo>> {
    let mut fee_infos: Vec<FeeInfo> = vec![]; 
    let mut done = false;
    let mut start_after_tick = lower_tick_index - 1;
    while !done {
      let tick_data: TickInfosResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pair_contract.clone(),
        msg: to_binary(&PairQueryMsg::TickInfos{
          start_after: Some(start_after_tick),
          // min(max_limit, tick_left)
          limit: Some(30.min((upper_tick_index - start_after_tick) as u32))
        })?,
      }))?;

      // if query data's length is not 30, then it queried all of the tick data that we need
      if tick_data.infos.len() != 30 {
        done = true;
      }
      
      for tick in tick_data.infos {
        if tick.tick_index > upper_tick_index {
          done = true;
        } else {
          fee_infos.push(FeeInfo {
            tick_index: tick.tick_index,
            last_fee_growth_0: tick.tick_info.last_fee_growth_0,
            last_fee_growth_1: tick.tick_info.last_fee_growth_1
          });
          start_after_tick = tick.tick_index;
        }
      }
    };

    Ok(fee_infos)
  }
}


impl<'a> LpContract<'a> {
  pub fn query(&self, deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
      QueryMsg::OwnerOf { token_id } => to_binary(&self.owner_of(deps, token_id)?),
      QueryMsg::Config {} => to_binary(&self.config(deps)?),
      QueryMsg::LiquidityInfo { token_id } => to_binary(&self.liquidity_info(deps, token_id)?), 
      QueryMsg::Tokens {
        owner,
        start_after,
        limit
      } => to_binary(&self.tokens(deps, owner, start_after, limit)?),
      QueryMsg::AllTokens {
        start_after,
        limit
      } => to_binary(&self.all_tokens(deps, start_after, limit)?),
      QueryMsg::Minter {} => to_binary(&self.minter(deps)?),
      QueryMsg::Reward { token_id } => to_binary(&self.reward(deps, token_id)?),
    }
  }
}