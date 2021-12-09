use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Map, Item};
use cosmwasm_std::{Uint128, Uint256, Decimal, Addr};

use wineswap::pair::TickInfo;
use wineswap::asset::AssetInfo;
use wineswap::new_int_key::NewInt32Key;

pub struct PairContract<'a> {
  pub config: Item<'a, Config>,
  pub tick_data: Map<'a, NewInt32Key, TickInfo>,
  pub current_tick_index: Item<'a, i32>,
  // price = Asset0 price as Asset1, Q128.128
  pub current_price_sqrt: Item<'a, Uint256>,
  pub cumulative_volume: Item<'a, [Uint128; 2]>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
  pub asset_infos: [AssetInfo; 2],
  pub tick_space: u16,
  pub fee_rate: Decimal,
  pub liquidity_token: Addr,
}

impl Default for PairContract<'static> {
  fn default() -> Self {
    Self::new(
      "config",
      "tick_data",
      "current_tick",
      "current_price_sqrt",
      "cumulative_volume",
    )
  }
}

impl<'a> PairContract<'a> {
  fn new(
    config_key: &'a str,
    tick_data_key: &'a str,
    current_tick_key: &'a str,
    current_price_sqrt_key: &'a str,
    cumulative_volume_key: &'a str,
  ) -> Self {
    Self {
      config: Item::new(config_key),
      tick_data: Map::new(tick_data_key),
      current_tick_index: Item::new(current_tick_key),
      current_price_sqrt: Item::new(current_price_sqrt_key),
      cumulative_volume: Item::new(cumulative_volume_key),
    }
  }
}