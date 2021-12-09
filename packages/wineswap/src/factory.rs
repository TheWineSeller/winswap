use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal};

use crate::asset::AssetInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
  pub owner: String,
  pub pair_code_id: u64,
  pub token_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
  pub owner: Addr,
  pub pair_code_id: u64,
  pub token_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
  UpdateConfig {
    owner: Option<String>,
    token_code_id: Option<u64>,
    pair_code_id: Option<u64>,
  },
  CreatePair {
    asset_infos: [AssetInfo; 2],
    pair_type: String,
    initial_price: Decimal,
  },
  AddPairType {
    type_name: String,
    tick_space: u16,
    fee_rate: Decimal,
  },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Pair {
      asset_infos: [AssetInfo; 2],
      pair_type: Option<String>,
    },
    Pairs {
      start_after: Option<AssetInfosWithType>,
      limit: Option<u32>,
    },
    PairType {
      type_name: String,
    },
    PairTypes {
      start_after: Option<String>,
      limit: Option<u32>,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PairTypeResponse {
  pub type_name: String,
  pub tick_space: u16,
  pub fee_rate: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PairsResponse {
  pub pairs: Vec<PairInfoWithType>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AssetInfosWithType {
  pub asset_infos: [AssetInfo; 2],
  pub pair_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PairInfoWithType {
  pub asset_infos: [AssetInfo; 2],
  pub contract_addr: Addr,
  pub liquidity_token: Addr,
  pub pair_type: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct PairInfo {
  pub asset_infos: [AssetInfo; 2],
  pub contract_addr: Addr,
  pub liquidity_token: Addr,
  pub pair_type: PairType,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct PairType {
  pub type_name: String,
  pub tick_space: u16,
  pub fee_rate: Decimal,
}

pub fn pair_key(asset_infos: &[AssetInfo; 2], pair_type: String) -> Vec<u8> {
  let mut asset_infos = asset_infos.to_vec();
  asset_infos.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

  [asset_infos[0].as_bytes(), asset_infos[1].as_bytes(), pair_type.as_bytes()].concat()
}