use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, StdResult, Storage, Decimal, Uint128};

use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};

pub use wineswap::lp_token::{ConfigResponse, Approval};

pub struct LpContract<'a> {
  pub config: Item<'a, ConfigResponse>,
  // token_count is for the index of the token_id
  // every mint event increase 1
  pub token_count: Item<'a, u64>,
  pub tokens: IndexedMap<'a, &'a str, LiquidityInfo, LiquidityIndexes<'a>>
}

impl Default for LpContract<'static> {
  fn default() -> Self {
    Self::new(
      "config",
      "num_tokens",
      "tokens",
      "tokens_owner",
    )
  }
}

impl<'a> LpContract<'a> {
  fn new(
    config_key: &'a str,
    token_count_key: &'a str,
    tokens_key: &'a str,
    tokens_owner_key: &'a str,
  ) -> Self {
    let indexes = LiquidityIndexes {
      owner: MultiIndex::new(token_owner_idx, tokens_key, tokens_owner_key),
    };
    Self {
      config: Item::new(config_key),
      token_count: Item::new(token_count_key),
      tokens: IndexedMap::new(tokens_key, indexes)
    }
  }

  pub fn token_count(&self, storage: &dyn Storage) -> StdResult<u64> {
    Ok(self.token_count.may_load(storage)?.unwrap_or_default())
  }

  pub fn increment_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
    let val = self.token_count(storage)? + 1;
    self.token_count.save(storage, &val)?;
    Ok(val)
  }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct FeeInfo {
  pub tick_index: i32,
  pub last_fee_growth_0: Decimal,
  pub last_fee_growth_1: Decimal,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct LiquidityInfo {
  pub owner: Addr,
  pub liquidity: Uint128,
  pub upper_tick_index: i32,
  pub lower_tick_index: i32,
  pub last_updated_fee_infos: Vec<FeeInfo>,
  pub approvals: Vec<Approval>,
}

pub struct LiquidityIndexes<'a> {
  pub owner: MultiIndex<'a, (Addr, Vec<u8>), LiquidityInfo>,
}

impl<'a> IndexList<LiquidityInfo> for LiquidityIndexes<'a> {
  fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<LiquidityInfo>> + '_> {
    let v: Vec<&dyn Index<LiquidityInfo>> = vec![&self.owner];
    Box::new(v.into_iter())
  }
}

pub fn token_owner_idx(d: &LiquidityInfo, k: Vec<u8>) -> (Addr, Vec<u8>) {
  (d.owner.clone(), k)
}