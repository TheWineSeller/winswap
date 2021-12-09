use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw0::Expiration;
use cosmwasm_std::{to_binary, BlockInfo, WasmMsg, CosmosMsg, StdResult, Addr, Binary, Uint128};

use crate::asset::Asset;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct InstantiateMsg {
  pub name: String,
  pub symbol: String,
  pub minter: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
  Transfer { recipient: String, token_id: String },

  Send { contract: String, token_id: String, msg: Binary },

  Approve {
    spender: String,
    token_id: String,
    expires: Option<Expiration>,
  },

  Revoke { spender: String, token_id: String },

  Burn { token_id: String },

  Mint {
    owner: String,
    /// liquidity will be calculated from pair contract.
    liquidity: Uint128,
    upper_tick_index: i32,
    lower_tick_index: i32
  },

  ClaimReward { token_id: String },

  UpdateLiquidity {
    token_id: String,
    amount: Uint128,
    add: bool
  },
}


/// Query Msgs
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
  OwnerOf {
    token_id: String,
  },

  Config {},

  LiquidityInfo {
    token_id: String,
  },
  
  Tokens {
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
  },

  AllTokens {
    start_after: Option<String>,
    limit: Option<u32>,
  },

  Minter {},

  Reward {
    token_id: String
  },
}



/// QueryResponses
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct OwnerOfResponse {
  pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ConfigResponse {
  pub name: String,
  pub symbol: String,
  pub minter: Addr,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct LiquidityInfoResponse {
  pub owner: Addr,
  pub approvals: Vec<Approval>,
  pub liquidity: Uint128,
  pub upper_tick_index: i32,
  pub lower_tick_index: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokensResponse {
  pub tokens: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MinterResponse {
  pub minter: String
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct RewardResponse {
  pub rewards: [Asset; 2]
}

/// receiver
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct LpReceiveMsg {
  pub sender: String,
  pub token_id: String,
  pub msg: Binary,
}

impl LpReceiveMsg {
  pub fn into_binary(self) -> StdResult<Binary> {
    let msg = ReceiverExecuteMsg::ReceiveLp(self);
    to_binary(&msg)
  }

  pub fn into_cosmos_msg<T: Into<String>>(self, contract_addr: T) -> StdResult<CosmosMsg> {
    let msg = self.into_binary()?;
    let execute = WasmMsg::Execute {
      contract_addr: contract_addr.into(),
      msg,
      funds: vec![],
    };
    Ok(execute.into())
  }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
enum ReceiverExecuteMsg {
    ReceiveLp(LpReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    /// Account that can transfer/send the token
    pub spender: Addr,
    /// When the Approval expires (maybe Expiration::never)
    pub expires: Expiration,
}

impl Approval {
  pub fn is_expired(&self, block: &BlockInfo) -> bool {
      self.expires.is_expired(block)
  }
}