
use cosmwasm_std::{to_binary, from_binary, Addr, CosmosMsg, Decimal, Decimal256, DepsMut, Env, MessageInfo,
  Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128, Uint256, WasmMsg};
use wineswap::lp_token::{InstantiateMsg as TokenInstantiateMsg, ExecuteMsg as TokenExecuteMsg};
use wineswap::new_int_key::NewInt32Key;
use protobuf::Message;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use wineswap::{
  pair::{InstantiateMsg, ExecuteMsg, TickInfo, TickIndexes, Cw20HookMsg},
  asset::{Asset, AssetInfo, TokenNumber},
};
use wineswap_math::{
  tick::{get_tick_from_price_sqrt, get_tick_price_sqrt, tick_to_tick_index, tick_index_to_tick, DENOMINATOR, MAX_TICK, MIN_TICK},
  liquidity::{compute_liquidity, get_token_amount_from_liquidity},
  swap::{compute_swap_tick}
};

use crate::response::MsgInstantiateContractResponse;
use crate::error::ContractError;
use crate::state::{Config, PairContract};

const TICK_RANGE_LIMIT: i32 = 500;
static DECIMAL_FRACTION: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);

impl<'a> PairContract<'a> {
  pub fn instantiate(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
  ) -> StdResult<Response> {
    if msg.tick_space == 0u16 {
      return Err(StdError::generic_err("Invalid tick space"));
    }

    if msg.fee_rate >= Decimal::one() {
      return Err(StdError::generic_err("Invalid fee rate"));
    }
    
    // save config
    let config = Config{
      asset_infos: msg.asset_infos,
      tick_space: msg.tick_space,
      fee_rate: msg.fee_rate,
      // temp addr
      liquidity_token: info.sender,
    };
  
    self.config.save(deps.storage, &config)?;

    let price_sqrt = msg.initial_price.clone().sqrt();

    // to Q128.128
    let price_sqrt: Uint256 = Decimal256::from_ratio(
      price_sqrt * DECIMAL_FRACTION,
      DECIMAL_FRACTION
    ) * DENOMINATOR;

    let max_price_sqrt = get_tick_price_sqrt(MAX_TICK + 1i32);
    let min_price_sqrt = get_tick_price_sqrt(MIN_TICK);

    if price_sqrt <= min_price_sqrt || price_sqrt >= max_price_sqrt {
      return Err(StdError::generic_err("Invalid price"));
    }

    // save price and tick
    self.current_price_sqrt.save(deps.storage, &price_sqrt)?;
    let tick = get_tick_from_price_sqrt(price_sqrt);
    let tick_index = tick_to_tick_index(tick, msg.tick_space);
    self.current_tick_index.save(deps.storage, &tick_index)?;

    // set initial data
    self.cumulative_volume.save(deps.storage, &[Uint128::zero(), Uint128::zero()])?;

    Ok(Response::new().add_submessage(SubMsg {
      msg: WasmMsg::Instantiate {
        admin: None,
        code_id: msg.token_code_id,
        label: "".to_string(),
        funds: vec![],
        msg: to_binary(&TokenInstantiateMsg{
          name: "wineswap concentrated liquidity token".to_string(),
          symbol: ("WINELP".to_string()),
          minter: env.contract.address.to_string(),
        })?,
      }
      .into(),
      gas_limit: None,
      id: 1,
      reply_on: ReplyOn::Success,
    }))
  }

  
  pub fn reply(&self, deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let data = msg.result.unwrap().data.unwrap();
    let res: MsgInstantiateContractResponse =
      Message::parse_from_bytes(data.as_slice()).map_err(|_| {
        StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
      })?;
    let liquidity_token = res.get_contract_address();
    let mut config = self.config.load(deps.storage)?;
    // update liquidity_token from the temp addr
    config.liquidity_token = deps.api.addr_validate(liquidity_token)?;

    self.config.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("liquidity_token_addr", liquidity_token))
  }

  pub fn execute(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
  ) -> Result<Response, ContractError> {
    match msg {
      ExecuteMsg::Receive(msg) => self.receive_cw20(deps, env, info, msg),
      ExecuteMsg::ProvideLiquidity {
        assets,
        token_id,
        tick_indexes
      } => self.provide(deps, env, info, assets, token_id, tick_indexes),
      ExecuteMsg::WithdrawLiquidity {
        token_id,
        amount,
      } => self.withdraw(deps, env, info, token_id, amount),
      ExecuteMsg::Swap {
        offer_asset,
        to,
        belief_price,
        max_slippage
      } => {
        if !offer_asset.is_native_token() {
          return Err(ContractError::Unauthorized {});
        }

        let to_addr = if let Some(to_addr) = to {
          Some(deps.api.addr_validate(&to_addr)?)
        } else {
            None
        };

        self.swap(deps, env, info.clone(), info.sender, offer_asset, to_addr, belief_price, max_slippage)
      },
      ExecuteMsg::ClaimReward { 
        token_id,
        rewards,
      } => self.claim(deps, env, info, token_id, rewards),
    }
  }
}

/// execute function
impl<'a> PairContract<'a> {
  pub fn receive_cw20(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
  ) -> Result<Response, ContractError> {
    let contract_addr = info.sender.clone();
  
    match from_binary(&cw20_msg.msg) {
      Ok(Cw20HookMsg::Swap {
        to,
        belief_price,
        max_slippage,
      }) => {
        // only asset contract can execute this message
        let mut authorized: bool = false;
        let config = self.config.load(deps.storage)?;
        
        for asset_info in config.asset_infos.iter() {
          if let AssetInfo::Token { contract_addr } = asset_info {
            if contract_addr == &info.sender.to_string() {
              authorized = true;
            }
          }
        }
  
        if !authorized {
          return Err(ContractError::Unauthorized {});
        }
  
        let to_addr = if let Some(to_addr) = to {
          Some(deps.api.addr_validate(to_addr.as_str())?)
        } else {
          None
        };
  
        self.swap(
          deps,
          env,
          info,
          Addr::unchecked(cw20_msg.sender),
          Asset {
            info: AssetInfo::Token {
              contract_addr: contract_addr.to_string(),
            },
            amount: cw20_msg.amount,
          },
          to_addr,
          belief_price,
          max_slippage,
        )
      },
      Err(err) => Err(ContractError::Std(err)),
    }
  }
  
  pub fn provide(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: [Asset; 2],
    token_id: Option<String>,
    tick_indexes: Option<TickIndexes>,
  ) -> Result<Response, ContractError> {
    // native sent balance check
    for asset in assets.iter() {
      asset.assert_sent_native_token_balance(&info)?;
    }

    let config = self.config.load(deps.storage)?;

    let asset_infos: [AssetInfo; 2] = config.asset_infos;
  
    let token_amount: [Uint128; 2] = [
      assets
        .iter()
        .find(|a| a.info.equal(&asset_infos[0]))
        .map(|a| a.amount)
        .expect("Wrong asset info is given"),
      assets
        .iter()
        .find(|a| a.info.equal(&asset_infos[1]))
        .map(|a| a.amount)
        .expect("Wrong asset info is given")
    ];

    let (lower_tick_index, upper_tick_index): (i32, i32);
    let additional_provide: bool;

    // when provide to already exist position
    if let Some(token_id) = token_id.clone() {
      additional_provide = true;
      // get liquidity info
      let liquidity_token = config.liquidity_token.to_string();
      let liquidity = self.get_liquidity_info(deps.querier, liquidity_token.clone(), token_id.clone())?;

      if info.sender != liquidity.owner {
        return Err(ContractError::Unauthorized {})
      }

      lower_tick_index = liquidity.lower_tick_index;
      upper_tick_index = liquidity.upper_tick_index;
    // when newly provide
    } else if let Some(tick_indexes) = tick_indexes {  
      additional_provide = false;
      lower_tick_index = tick_indexes.lower_tick_index;
      upper_tick_index = tick_indexes.upper_tick_index;
      // tick check
      if lower_tick_index > upper_tick_index {
        return Err(ContractError::InvalidTickRange {})
      }
  
      if tick_index_to_tick(upper_tick_index, config.tick_space) > MAX_TICK 
      || tick_index_to_tick(lower_tick_index, config.tick_space) < MIN_TICK {
        return Err(ContractError::InvalidTickRange {})
      }
  
      if upper_tick_index - lower_tick_index > TICK_RANGE_LIMIT {
        return Err(ContractError::TickRangeLimit {})
      }
    } else {
      return Err(ContractError::ProvideOptionError {})
    }

    // Q128.128
    let current_price_sqrt = self.current_price_sqrt.load(deps.storage)?;

    // get liquidity from the given amount
    let liquidity = compute_liquidity(
      token_amount[0],
      token_amount[1],
      current_price_sqrt,
      upper_tick_index,
      lower_tick_index,
      config.tick_space,
    );
  
    // get amount from the liquidity
    let (token0_provide_amount, token1_provide_amount)
      = get_token_amount_from_liquidity(
        upper_tick_index,
        lower_tick_index,
        config.tick_space,
        current_price_sqrt,
        liquidity
      );

    let provide_amount = [token0_provide_amount, token1_provide_amount];
    
    let mut messages: Vec<CosmosMsg> = vec![];
    for (i, asset_info) in asset_infos.iter().enumerate() {
      // if token, get token by transfer_from
      if let AssetInfo::Token { contract_addr, .. } = &asset_info {
        if !provide_amount[i].is_zero() {
          messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
              owner: info.sender.to_string(),
              recipient: env.contract.address.to_string(),
              amount: provide_amount[i],
            })?,
            funds: vec![],
          }));
        }
      } else {
        // If native asset provided more than need, return asset
        let return_amount = token_amount[i].checked_sub(provide_amount[i])?;
        if return_amount != Uint128::zero() {
          let return_asset = Asset{
            info: asset_info.clone(),
            amount: return_amount
          };
          if !return_asset.deduct_tax(&deps.querier)?.amount.is_zero() {
            messages.push(return_asset.into_msg(&deps.querier, info.sender.clone())?);
          }
        }
      }
    }

    if liquidity.is_zero() {
      return Err(ContractError::ZeroLiquidity {})
    }
  
    // update ticks
    for i in lower_tick_index..(upper_tick_index + 1) {
      self.tick_data
        .update(deps.storage, NewInt32Key::new(i), |tick| match tick {
          // if tick data exist, update
          Some(tick) => {
            let mut new_tick = tick.clone();
            // update liquidity
            new_tick.total_liquidity = tick.total_liquidity + liquidity;
            Ok(new_tick)
          }
          // if not exist make new
          None => {
            let tick = TickInfo {
              last_fee_growth_0: Decimal::zero(),
              last_fee_growth_1: Decimal::zero(),
              total_liquidity: liquidity,
            };
            Ok(tick)
          }
          _ => Err(ContractError::UpdateFail {})
        })?;
    }

    if additional_provide {
      // have to claim reward first
      messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.liquidity_token.to_string(),
        msg: to_binary(&TokenExecuteMsg::ClaimReward {
          token_id: token_id.clone().unwrap(),
        })?,
        funds: vec![],
      }));
      // update liquidity
      messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.liquidity_token.to_string(),
        msg: to_binary(&TokenExecuteMsg::UpdateLiquidity {
          token_id: token_id.clone().unwrap(),
          amount: liquidity,
          add: true
        })?,
        funds: vec![],
      }));
    } else {
      // mint Lp token
      messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.liquidity_token.to_string(),
        msg: to_binary(&TokenExecuteMsg::Mint {
          owner: info.sender.clone().into_string(),
          liquidity,
          upper_tick_index,
          lower_tick_index
        })?,
        funds: vec![],
      }));
    }

    Ok(Response::new().add_messages(messages)
      .add_attribute("action", "provide_liquidity")
      .add_attribute("sender", info.sender.to_string())
      .add_attribute("provide_assets", format!("{}, {}", 
        Asset {
          info: asset_infos[0].clone(),
          amount: provide_amount[0]
        },
        Asset { 
          info: asset_infos[1].clone(),
          amount: provide_amount[1]
        }
      ))
      .add_attribute("liquidity", liquidity.to_string())
      .add_attribute(
        "token_id",
      if let Some(token_id) = token_id {
        token_id
      } else {
        "new".to_string()
      }),
    )
  }

  pub fn withdraw(
    &self,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
    amount: Option<Uint128>
  ) -> Result<Response, ContractError> {
    let config = self.config.load(deps.storage)?;
    let liquidity_token = config.liquidity_token.to_string();
    let partial_withdraw: bool;
    let withdraw_amount: Uint128;
    let liquidity = self.get_liquidity_info(deps.querier, liquidity_token.clone(), token_id.clone())?;

    // partial withdraw
    if let Some(amount) = amount {
      partial_withdraw = true;
      withdraw_amount = amount;
    } else {
      partial_withdraw = false;
      withdraw_amount = liquidity.liquidity
    }

    if info.sender != liquidity.owner {
      return Err(ContractError::Unauthorized {})
    }

    let asset_infos = config.asset_infos;
    let current_price_sqrt = self.current_price_sqrt.load(deps.storage)?;

    // caculate withdraw amount
    let (token0_amount, token1_amount) = get_token_amount_from_liquidity(
      liquidity.upper_tick_index,
      liquidity.lower_tick_index,
      config.tick_space,
      current_price_sqrt,
      withdraw_amount
    );

    // update ticks
    for i in liquidity.lower_tick_index..(liquidity.upper_tick_index + 1) {
      self.tick_data
        .update(deps.storage, NewInt32Key::new(i), |tick| match tick {
          Some(tick) => {
            let mut new_tick = tick.clone();
            // update liquidity
            new_tick.total_liquidity = tick.total_liquidity.checked_sub(withdraw_amount)?;
            Ok(new_tick)
          }
          None => {
            // never get this.
            Err(ContractError::UpdateFail {})
          }
        })?;
    }

    let assets = [
      Asset{
        info: asset_infos[0].clone(),
        amount: token0_amount,
      },
      Asset {
        info: asset_infos[1].clone(),
        amount: token1_amount,
      },
    ];

    let mut messages: Vec<CosmosMsg> = vec![];

    // refund assets
    for asset in assets.clone() {
      match asset.clone().info {
        AssetInfo::Token { .. } => {
          if !asset.amount.is_zero() {
            messages.push(asset.clone().into_msg(&deps.querier, liquidity.owner.clone())?);
          }
        },
        AssetInfo::NativeToken { .. } => {
          if !asset.deduct_tax(&deps.querier)?.amount.is_zero() {
            messages.push(asset.clone().into_msg(&deps.querier, liquidity.owner.clone())?);
          }
        }
      }
    }

    // claim reward
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
      contract_addr: config.liquidity_token.to_string(),
      msg: to_binary(&TokenExecuteMsg::ClaimReward {
        token_id: token_id.clone(),
      })?,
      funds: vec![],
    }));

    if partial_withdraw  {
      // update liquidity
      messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: liquidity_token,
        msg: to_binary(&TokenExecuteMsg::UpdateLiquidity { 
          token_id: token_id.clone(),
          amount: withdraw_amount,
          add: false,
        })?,
        funds: vec![],
      }));
    } else {
      // burn liquidity
      messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: liquidity_token,
        msg: to_binary(&TokenExecuteMsg::Burn { token_id: token_id.clone() })?,
        funds: vec![],
      }));
    }

    Ok(Response::new().add_messages(messages)
      .add_attribute("action", "withdraw_liquidity")
      .add_attribute("sender", info.sender.to_string())
      .add_attribute("withdraw_assets", format!("{}, {}", assets[0], assets[1]))
      .add_attribute("liquidity_token_id", token_id),
    )
  }

  pub fn swap(
    &self,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    sender:Addr,
    offer_asset: Asset,
    to: Option<Addr>,
    belief_price: Option<Decimal>,
    max_slippage: Option<Decimal>
  ) -> Result<Response, ContractError> {
    // native sent balance check
    offer_asset.assert_sent_native_token_balance(&info)?;

    let config = self.config.load(deps.storage)?;
    let tick_index = self.current_tick_index.load(deps.storage)?;

    let asset_infos = config.asset_infos;
    let current_price_sqrt = self.current_price_sqrt.load(deps.storage)?;

    let mut remain = offer_asset.amount.clone();
    let mut tick_index_temp = tick_index.clone();
    let mut price_sqrt_temp = current_price_sqrt.clone();
    let mut total_return_amount = Uint128::zero();
    let mut total_commission_amount = Uint128::zero();


    let offer_token: TokenNumber;
    let return_token_info: AssetInfo;

    if offer_asset.info.equal(&asset_infos[0]) {
      offer_token = TokenNumber::Token0;
      return_token_info = asset_infos[1].clone();
    } else if offer_asset.info.equal(&asset_infos[1]){
      offer_token = TokenNumber::Token1;
      return_token_info = asset_infos[0].clone();
    } else {
      return Err(ContractError::AssetMismatch {});
    }

    while remain > Uint128::zero() {
      let tick_data = match self.tick_data.may_load(deps.storage, NewInt32Key::from(tick_index_temp))? {
        Some(tick_data) => tick_data,
        None => return Err(ContractError::CanNotSwap {})
      };

      if tick_data.total_liquidity.is_zero() {
        return Err(ContractError::CanNotSwap {}) 
      }
      

      // compute swap
      let (offer_amount, return_amount, commission_amount, next_price_sqrt, next_tick_index) 
        = compute_swap_tick(tick_index_temp, config.tick_space, price_sqrt_temp, tick_data.total_liquidity, &offer_token, remain, config.fee_rate);

      // update
      // update commission
      self.tick_data.update(deps.storage, NewInt32Key::new(tick_index_temp), |tick| match tick {
        Some(tick) => {
          let mut new_tick = tick.clone();
          match offer_token {
            TokenNumber::Token0 => {
              new_tick.last_fee_growth_1 = tick.last_fee_growth_1 
                + Decimal::from_ratio(commission_amount, tick.total_liquidity);
            },
            TokenNumber::Token1 => {
              new_tick.last_fee_growth_0 = tick.last_fee_growth_0 
                + Decimal::from_ratio(commission_amount, tick.total_liquidity);
            }
          }
          Ok(new_tick)
        }
        None => {
          // never get this.
          Err(ContractError::UpdateFail {})
        }
      })?;

      remain = remain.checked_sub(offer_amount)?; 
      total_return_amount += return_amount;
      total_commission_amount += commission_amount;
      tick_index_temp = next_tick_index;
      price_sqrt_temp = next_price_sqrt;
    }

    // update volume fee
    let mut volume = self.cumulative_volume.load(deps.storage)?;

    match offer_token {
      TokenNumber::Token0 => {
        volume[0] = volume[0].wrapping_add(offer_asset.amount);
        volume[1] = volume[1].wrapping_add(total_return_amount);
      },
      TokenNumber::Token1 => {
        volume[1] = volume[1].wrapping_add(offer_asset.amount);
        volume[0] = volume[0].wrapping_add(total_return_amount);
      }
    }

    self.cumulative_volume.save(deps.storage, &volume)?;

    let user_return_amount = total_return_amount.checked_sub(total_commission_amount)?;

    // slippage protection
    if let (Some(max_slippage), Some(belief_price)) = (max_slippage, belief_price) {
      // min_return = expected_return * (1 - max_slippage)
      // = offer_amount / belief_price * (1 - max_slippage)
      let min_return = Decimal::from_ratio(
        offer_asset.amount * DECIMAL_FRACTION,
        belief_price * DECIMAL_FRACTION
      ) * Uint128::from(1u128) 
      * (Decimal::one() - max_slippage);

      if min_return > user_return_amount {
        return Err(ContractError::MaxSlippage {} )
      }
    };

    // update state
    self.current_price_sqrt.save(deps.storage, &price_sqrt_temp)?;
    self.current_tick_index.save(deps.storage, &tick_index_temp)?;

    let return_asset = Asset {
      info: return_token_info.clone(),
      amount: user_return_amount,
    };

    let receiver = to.unwrap_or_else(|| sender.clone());

    let tax_amount = return_asset.compute_tax(&deps.querier)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    if !user_return_amount.is_zero() {
        messages.push(return_asset.into_msg(&deps.querier, receiver.clone())?);
    }

    Ok(Response::new().add_messages(messages)
      .add_attribute("action", "swap")
      .add_attribute("sender", sender.to_string())
      .add_attribute("receiver", receiver.to_string())
      .add_attribute("offer_asset", offer_asset.info.to_string())
      .add_attribute("return_asset", return_token_info.to_string())
      .add_attribute("offer_amount", offer_asset.amount.to_string())
      .add_attribute("return_amount", user_return_amount.to_string())
      .add_attribute("tax_amount", tax_amount.to_string())
      .add_attribute("commission_amount", total_commission_amount.to_string())
    )
  }

  pub fn claim(
    &self,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
    rewards: [Asset; 2],
  ) -> Result<Response, ContractError> {
    let config = self.config.load(deps.storage)?;

    let liquidity_token = config.liquidity_token.to_string();
    // only liquidity token contract can execute
    if liquidity_token.to_string() != info.sender.to_string() {
      return Err(ContractError::Unauthorized {})
    }

    let liquidity = self.get_liquidity_info(deps.querier, liquidity_token.to_string(), token_id)?;
    let owner = liquidity.owner;
    
    let mut messages: Vec<CosmosMsg> = vec![];

    for reward in rewards.clone() {
      match reward.clone().info {
        AssetInfo::Token { .. } => {
          if !reward.amount.is_zero() {
            messages.push(reward.clone().into_msg(&deps.querier, owner.clone())?);
          }
        },
        AssetInfo::NativeToken { .. } => {
          if !reward.deduct_tax(&deps.querier)?.amount.is_zero() {
            messages.push(reward.clone().into_msg(&deps.querier, owner.clone())?);
          }
        }
      }
    }
    

    Ok(Response::new().add_messages(messages)
      .add_attribute("action", "claim_reward")
      .add_attribute("sender", info.sender.to_string())
      .add_attribute("owner", owner.to_string())
      .add_attribute("claim_amount", format!("{}, {}", rewards[0], rewards[1]))
    )
  }
}