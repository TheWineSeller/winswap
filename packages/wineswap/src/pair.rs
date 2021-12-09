use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Decimal256, Uint128};
use cw20::Cw20ReceiveMsg;

use crate::asset::{Asset, AssetInfo};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
  pub asset_infos: [AssetInfo; 2],
  pub tick_space: u16,
  pub fee_rate: Decimal,
  pub liquidity_token: Addr,
}


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct TickInfo {
  pub last_fee_growth_0: Decimal,
  pub last_fee_growth_1: Decimal,
  pub total_liquidity: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct InstantiateMsg {
  pub asset_infos: [AssetInfo; 2],
  pub token_code_id: u64,
  /// Initial price. Asset0 price as Asset1
  pub initial_price: Decimal,
  // 0 < tick_space
  pub tick_space: u16,
  pub fee_rate: Decimal,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TickIndexes {
  // real tick = tick_index * tick_space
  pub upper_tick_index: i32,
  pub lower_tick_index: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
  Receive(Cw20ReceiveMsg),

  ProvideLiquidity {
    assets: [Asset; 2],
    // when provide to exist position put token_id
    token_id: Option<String>,
    // when make new position put tick_indexes
    tick_indexes: Option<TickIndexes>
  },

  WithdrawLiquidity  {
    token_id: String,
    amount: Option<Uint128>
  },

  Swap { 
    offer_asset: Asset,
    to: Option<String>,
    belief_price: Option<Decimal>,
    max_slippage: Option<Decimal>,
  },

  ClaimReward {
    token_id: String,
    rewards: [Asset; 2],
  },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
  Swap {
    to: Option<String>,
    belief_price: Option<Decimal>,
    max_slippage: Option<Decimal>,
  },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LpHookMsg {
  WithdrawBurn {},
}

/// Query Msgs
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
  PairInfo {},
  
  TickInfo {
    tick_index: i32
  },

  TickInfos {
    start_after: Option<i32>,
    limit: Option<u32>,
  },

  ProvideCalculation { 
    asset: Asset,
    upper_tick_index: i32,
    lower_tick_index: i32
  },

  WithdrawCalculation { 
    token_id: String
  },

  Simulation { offer_asset: Asset },

  ReverseSimulation { ask_asset: Asset },

  CumulativeVolume {},
}



/// QueryResponses
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct PairInfoResponse {
  pub liquidity_token: String,
  pub asset_infos: [AssetInfo; 2],
  pub tick_space: u16,
  pub fee_rate: Decimal,
  pub price: Decimal256,
  pub current_tick_index: i32
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ProvideCalculationResponse {
  pub asset: Asset,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct WithdrawCalculationResponse {
  pub assets: [Asset; 2],
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TickInfoResponse {
  pub tick_index: i32,
  pub tick_info: TickInfo
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TickInfosResponse {
  pub infos: Vec<TickInfoResponse>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SimulationResponse {
  pub return_amount: Uint128,
  pub commission_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ReverseSimulationResponse {
  pub offer_amount: Uint128,
  pub commission_amount: Uint128,
}