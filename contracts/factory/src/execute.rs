use cosmwasm_std::{to_binary, Binary, Decimal, DepsMut, Env, MessageInfo, Order, QueryRequest, Reply, ReplyOn, Response,
  StdError, StdResult, SubMsg, WasmMsg, WasmQuery};

use wineswap::factory::{Config, InstantiateMsg, ExecuteMsg, PairInfo, PairType};
use wineswap::pair::{Config as PairConfig, InstantiateMsg as PairInstantiateMsg};
use wineswap::asset::AssetInfo;
use protobuf::Message;

use crate::state::{pair_key, asset_infos_key, FactoryContract, TmpPairInfo};
use crate::response::MsgInstantiateContractResponse;
use crate::error::ContractError;

impl<'a> FactoryContract<'a> {
  pub fn instantiate(
    &self,
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg
  ) -> StdResult<Response> {
    let config = Config {
      owner: deps.api.addr_validate(&msg.owner)?,
      pair_code_id: msg.pair_code_id,
      token_code_id: msg.token_code_id,
    };

    self.config.save(deps.storage, &config)?;

    Ok(Response::new())
  }

  pub fn reply(&self, deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let tmp_pair_info = self.temp_pair_info.load(deps.storage)?;

    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(msg.result.unwrap().data.unwrap().as_slice()).map_err(|_| {
          StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;

    let pair_contract = res.get_contract_address();
    let pair_config: PairConfig = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
      contract_addr: pair_contract.to_string(),
      key: Binary::from("config".as_bytes()),
    }))?;
    let liquidity_token = pair_config.liquidity_token;

    self.pairs.save(
        deps.storage,
        tmp_pair_info.pair_key,
        &PairInfo {
          liquidity_token: liquidity_token.clone(),
          contract_addr: deps.api.addr_validate(pair_contract)?,
          asset_infos: tmp_pair_info.asset_infos,
          pair_type: tmp_pair_info.pair_type
        }
    )?;

    Ok(Response::new()
     .add_attribute("pair_contract_addr", pair_contract)
     .add_attribute("liquidity_token_addr", liquidity_token.to_string())
    )
}

  pub fn execute(
    &self, 
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
  ) -> Result<Response, ContractError> {
    match msg{
      ExecuteMsg::UpdateConfig {
        owner,
        token_code_id,
        pair_code_id,
      } => self.update_config(deps, env, info, owner, token_code_id, pair_code_id), 
      ExecuteMsg::CreatePair {
        asset_infos,
        pair_type,
        initial_price,
      } => self.create_pair(deps, env, info, asset_infos, pair_type, initial_price),
      ExecuteMsg::AddPairType {
        type_name,
        tick_space,
        fee_rate,
      } => self.add_pair_type(deps, env, info, type_name, tick_space, fee_rate),
    }
  }
}

/// execute function
impl<'a> FactoryContract<'a> {
  pub fn update_config(
    &self,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: Option<String>,
    token_code_id: Option<u64>,
    pair_code_id: Option<u64>,
  ) -> Result<Response, ContractError> {
    let mut config: Config = self.config.load(deps.storage)?;
    
    if info.sender != config.owner {
      return Err(ContractError::Unauthorized {})
    }

    if let Some(owner) = owner {
      config.owner = deps.api.addr_validate(&owner)?;
    }

    if let Some(token_code_id) = token_code_id {
      config.token_code_id = token_code_id;
    }

    if let Some(pair_code_id) = pair_code_id {
      config.pair_code_id = pair_code_id;
    }

    self.config.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
  }

  pub fn create_pair(
    &self,
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    asset_infos: [AssetInfo; 2],
    pair_type: String,
    initial_price: Decimal,
  ) -> Result<Response, ContractError> {
    let key = pair_key(&asset_infos, pair_type.clone());

    if let Ok(Some(_)) = self.pairs.may_load(deps.storage, key.clone()) {
      return Err(ContractError::PairExists {})
    }

    let type_data = self.pair_type.load(deps.storage, pair_type.as_bytes().to_vec())?; 

    self.temp_pair_info.save(
      deps.storage,
      &TmpPairInfo {
        pair_key: key,
        asset_infos: asset_infos.clone(),
        pair_type: type_data.clone()
      }
    )?;

    let config = self.config.load(deps.storage)?;
    Ok(Response::new()
      .add_attribute("action", "create_pair")
      .add_attribute("pair", &format!("{}-{}", asset_infos[0], asset_infos[1]))
      .add_submessage(SubMsg {
        id: 1,
        gas_limit: None,
        msg: WasmMsg::Instantiate {
          code_id: config.pair_code_id,
          funds: vec![],
          admin: None,
          label: "".to_string(),
          msg: to_binary(&PairInstantiateMsg {
            asset_infos,
            token_code_id: config.token_code_id,
            initial_price,
            tick_space: type_data.tick_space,
            fee_rate: type_data.fee_rate
          })?
        }.into(),
        reply_on: ReplyOn::Success
      }))
  }

  pub fn add_pair_type(
    &self,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    type_name: String,
    tick_space: u16,
    fee_rate: Decimal
  ) -> Result<Response, ContractError> {
    let config = self.config.load(deps.storage)?;
    if info.sender != config.owner {
      return Err(ContractError::Unauthorized {})
    }

    if fee_rate < Decimal::zero() || fee_rate > Decimal::one() {
      return Err(ContractError::InvalidFeeRate {})
    }

    if tick_space == 0 {
      return Err(ContractError::InvalidTickSpace {})
    }

    let key = type_name.as_bytes().to_vec();

    if let Ok(Some(_)) = self.pair_type.may_load(deps.storage, key.clone()) {
      return Err(ContractError::PairTypeExists {})
    }

    // add pair type
    self.pair_type.save(deps.storage, key, &PairType {
      type_name: type_name.clone(),
      tick_space,
      fee_rate
    })?;

    Ok(Response::new()
      .add_attribute("action", "add_pair_type")
      .add_attribute("type_name", type_name)
      .add_attribute("tick_space", tick_space.to_string())
      .add_attribute("fee_rate", fee_rate.to_string())
    )
  }
}

impl<'a> FactoryContract<'a> {
  pub fn ust_pair_existence(&self, deps: &DepsMut, asset_info: AssetInfo) -> bool {
    let uusd = AssetInfo::NativeToken{ denom: "uusd".to_string() };
    let asset_infos_key = asset_infos_key(&[uusd, asset_info]);
    let pks: Vec<_> = self.pairs
      .idx
      .asset_infos
      .prefix(asset_infos_key)
      .keys(deps.storage, None, None, Order::Ascending)
      .take(1)
      .collect();

    if pks.len() == 0{
      return false
    } else {
      return true
    }
  }
}