use cosmwasm_std::{to_binary, Binary, Deps, StdResult, Order};
use cw_storage_plus::Bound;
use wineswap::factory::{AssetInfosWithType, Config, PairInfoWithType, PairType, QueryMsg};
use wineswap::asset::AssetInfo;

use crate::state::{pair_key, asset_infos_key, FactoryContract};


const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

impl<'a> FactoryContract<'a> {
  fn config(&self, deps: Deps) -> StdResult<Config> {
    self.config.load(deps.storage)
  }
  fn pair(&self, deps: Deps, asset_infos: [AssetInfo; 2], pair_type: Option<String>)  -> StdResult<Vec<PairInfoWithType>>{
    if let Some(pair_type) = pair_type {
      let key = pair_key(&asset_infos, pair_type.clone());
      let pair = self.pairs.load(deps.storage, key)?;
      return Ok([PairInfoWithType {
        asset_infos: pair.asset_infos,
        contract_addr: pair.contract_addr,
        liquidity_token: pair.liquidity_token,
        pair_type,
      }].to_vec())
    } else {
      let asset_infos_key = asset_infos_key(&asset_infos);
      let pks: Vec<_> = self.pairs
        .idx
        .asset_infos
        .prefix(asset_infos_key)
        .keys(deps.storage, None, None, Order::Ascending)
        // will not exceed 30, I'll change owner addr to factory addr after add pair_type
        .take(30)
        .collect();

      let res: Vec<PairInfoWithType> = pks.iter().map(|v| {
        let pair = self.pairs.load(deps.storage, v.clone()).unwrap();
        return PairInfoWithType {
          asset_infos: pair.asset_infos,
          contract_addr: pair.contract_addr,
          liquidity_token: pair.liquidity_token,
          pair_type: pair.pair_type.type_name,
        }
      }).collect();

      return Ok(res)
    }
  }
  fn pairs(&self, deps: Deps, start_after: Option<AssetInfosWithType>, limit: Option<u32>) -> StdResult<Vec<PairInfoWithType>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = if let Some(start_after) = start_after {
      Some(Bound::exclusive(pair_key(&start_after.asset_infos, start_after.pair_type)))
    } else {
      None
    };

    let pairs: Vec<PairInfoWithType> = self.pairs
      .range(deps.storage, start, None, Order::Ascending)
      .take(limit)
      .map(|item| {
        let(_, v) = item.unwrap();
        PairInfoWithType {
          asset_infos: v.asset_infos,
          contract_addr: v.contract_addr,
          liquidity_token: v.liquidity_token,
          pair_type: v.pair_type.type_name
        }
      })
      .collect();
    
    Ok(pairs)
  }
  fn pair_type(&self, deps: Deps, type_name: String) -> StdResult<PairType> {
    self.pair_type.load(deps.storage, type_name.as_bytes().to_vec())
  }

  fn pair_types(&self, deps: Deps, start_after: Option<String>, limit: Option<u32>) -> StdResult<Vec<PairType>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    return self.pair_type
      .range(deps.storage, start, None, Order::Ascending)
      .take(limit)
      .map(|item| item.map(|(_, v)| v))
      .collect();
  }
}

impl<'a> FactoryContract<'a> {
  pub fn query(&self, deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
      QueryMsg::Config {} => to_binary(&self.config(deps)?),
      QueryMsg::Pair { asset_infos, pair_type } => to_binary(&self.pair(deps, asset_infos, pair_type)?),
      QueryMsg::Pairs { start_after, limit } => to_binary(&self.pairs(deps, start_after, limit)?),
      QueryMsg::PairType { type_name } => to_binary(&self.pair_type(deps, type_name)?),
      QueryMsg::PairTypes { start_after, limit } => to_binary(&self.pair_types(deps, start_after, limit)?),
    }
  }
}
