use cosmwasm_std::{to_binary, Binary, Deps, Order, QuerierWrapper, QueryRequest, StdError, StdResult,
  Uint128, WasmQuery};
use cw_storage_plus::Bound;
use wineswap::new_int_key::NewInt32Key;

use wineswap::pair::{PairInfoResponse, ProvideCalculationResponse, QueryMsg, ReverseSimulationResponse,
  SimulationResponse, TickInfoResponse, TickInfosResponse, WithdrawCalculationResponse};
use wineswap::asset::{Asset, AssetInfo, TokenNumber};
use wineswap::lp_token::{LiquidityInfoResponse, QueryMsg::LiquidityInfo};
use wineswap_math::tick::{get_tick_price_sqrt};
use wineswap_math::liquidity::{compute_token_liquidity, get_token_amount_from_liquidity};
use wineswap_math::swap::{compute_swap_tick, compute_swap_tick_reverse};
use wineswap_math::price::price_sqrt_to_price;
use crate::state::PairContract;


const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

impl<'a> PairContract<'a> {
  fn pair_info(&self, deps: Deps) -> StdResult<PairInfoResponse> {
    let config = self.config.load(deps.storage)?;
    let price_sqrt = self.current_price_sqrt.load(deps.storage)?;
    let current_tick_index = self.current_tick_index.load(deps.storage)?;
    // price_sqrt (Q128.128) to price (readable format)
    let price = price_sqrt_to_price(price_sqrt);

    Ok(PairInfoResponse {
      liquidity_token: config.liquidity_token.to_string(),
      asset_infos: config.asset_infos,
      tick_space: config.tick_space,
      fee_rate: config.fee_rate,
      price,
      current_tick_index
    })
  }

  fn tick_info(&self, deps: Deps, tick_index: i32) -> StdResult<TickInfoResponse> {
    let tick_info = self.tick_data.load(deps.storage, NewInt32Key::from(tick_index))?;
    Ok(TickInfoResponse {
      tick_index: tick_index,
      tick_info: tick_info
    })
  }

  fn tick_infos(&self, deps: Deps, start_after: Option<i32>, limit: Option<u32>) -> StdResult<TickInfosResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let start = if let Some(start_after) = start_after {
      Some(Bound::exclusive(NewInt32Key::new(start_after)))
    } else {
      None
    };

    let ticks = self.tick_data
      .range(deps.storage, start, None, Order::Ascending)
      .take(limit)
      .map(|item| {
        let (k, v) = item.unwrap();
        TickInfoResponse {
          tick_index: NewInt32Key::from(k).into(),
          tick_info: v
        }
      })
      .collect();

    Ok(TickInfosResponse {
      infos: ticks
    })
  }

  fn provide_calculation(
    &self, deps: Deps,
    asset: Asset,
    upper_tick_index: i32,
    lower_tick_index: i32
  ) -> StdResult<ProvideCalculationResponse> {
    let config = self.config.load(deps.storage)?;
    let asset_infos: [AssetInfo; 2] = config.asset_infos;
    let price_high_sqrt = get_tick_price_sqrt((upper_tick_index + 1) * i32::from(config.tick_space));
    let price_low_sqrt = get_tick_price_sqrt(lower_tick_index * i32::from(config.tick_space));
    let price_sqrt = self.current_price_sqrt.load(deps.storage)?;

    let result_asset: Asset;

    if asset.info.equal(&asset_infos[0]) {
      // asset0 is given
      // case1. out of price range, (price is higher than price_high)
      if price_high_sqrt < price_sqrt {
        // price_high must be higher than current price, 0 liqudity
        return Err(StdError::generic_err("asset0 must be 0 amount"));
      // case2. out of price range, (price is lower than price_low)
      } else if price_sqrt < price_low_sqrt {
        // only amount0 provide, amount1 = 0 
        result_asset = Asset {
          info: asset_infos[1].clone(),
          amount: Uint128::zero()
        };
      // case3. price in price range
      } else {
        // use current price as lower price
        let price_low_sqrt = price_sqrt;
        // compute liquidity
        let liquidity = compute_token_liquidity(TokenNumber::Token0, asset.amount, price_high_sqrt, price_low_sqrt);
        // get asset1 amount
        let (_, asset1_amount) = get_token_amount_from_liquidity(
          upper_tick_index,
          lower_tick_index,
          config.tick_space,
          price_sqrt,
          liquidity
        );
        result_asset = Asset {
          info: asset_infos[1].clone(),
          amount: asset1_amount
        };
      }
    } else if asset.info.equal(&asset_infos[1]){
      // asset1 is given
      // case1. out of price range, (price is higher than price_high
      if price_high_sqrt < price_sqrt {
        // only amount1 provide, amount0 = 0
        result_asset = Asset {
          info: asset_infos[0].clone(),
          amount: Uint128::zero()
        };
      // case2. out of price range, (price is lower than price_low)
      } else if price_sqrt < price_low_sqrt {
        // price_low must be lower than current price, 0 liquidity
        return Err(StdError::generic_err("asset1 must be 0 amount"));
      // case3. price in price range
      } else  {
        // use current pirce as higher price
        let price_high_sqrt = price_sqrt;
        // compute liquidity
        let liquidity = compute_token_liquidity(TokenNumber::Token1, asset.amount, price_high_sqrt, price_low_sqrt);
        // get asset0 amount 
        let (asset0_amount, _) = get_token_amount_from_liquidity(
          upper_tick_index,
          lower_tick_index,
          config.tick_space,
          price_sqrt,
          liquidity
        );
        result_asset = Asset {
          info: asset_infos[0].clone(),
          amount: asset0_amount
        };
      }
    } else {
      return Err(StdError::generic_err("Token missmatched"));
    }

    return Ok(ProvideCalculationResponse { asset: result_asset })
  }

  fn withdraw_calculation(
    &self, deps: Deps,
    token_id: String
  ) -> StdResult<WithdrawCalculationResponse> {
    let config = self.config.load(deps.storage)?;
    let asset_infos: [AssetInfo; 2] = config.asset_infos;
    let liquidity_info = self.get_liquidity_info(deps.querier, config.liquidity_token.to_string(), token_id)?;
  
    let price_sqrt = self.current_price_sqrt.load(deps.storage)?;

    let (amount0, amount1) = get_token_amount_from_liquidity(
      liquidity_info.upper_tick_index,
      liquidity_info.lower_tick_index,
      config.tick_space,
      price_sqrt,
      liquidity_info.liquidity
    );

    let assets = [
      Asset {
        info: asset_infos[0].clone(),
        amount: amount0
      },
      Asset {
        info: asset_infos[1].clone(),
        amount: amount1
      },
    ];

    return Ok(WithdrawCalculationResponse { assets })
  }

  fn swap_simulation(&self, deps: Deps, asset: Asset) -> StdResult<SimulationResponse> {
    let price_sqrt = self.current_price_sqrt.load(deps.storage)?;
    let tick_index = self.current_tick_index.load(deps.storage)?;
    let config = self.config.load(deps.storage)?;

    let mut remain = asset.amount.clone();
    let mut tick_index_temp = tick_index.clone();
    let mut price_sqrt_temp = price_sqrt.clone();
    let mut total_return_amount = Uint128::zero();
    let mut total_commission_amount = Uint128::zero();

    let offer_token: TokenNumber;

    let asset_infos: [AssetInfo; 2] = config.asset_infos;

    if asset.info.equal(&asset_infos[0]) {
      offer_token = TokenNumber::Token0;
    } else if asset.info.equal(&asset_infos[1]){
      offer_token = TokenNumber::Token1;
    } else {
      return Err(StdError::generic_err("Token missmatched"));
    }
    
    while remain > Uint128::zero() {
      let tick_data = match self.tick_data.may_load(deps.storage, NewInt32Key::from(tick_index_temp))? {
        Some(tick_data) => tick_data,
        None => return Err(StdError::generic_err("Can't swap"))
      };
      

      // compute swap
      let (offer_amount_, return_amount, commission_amount, next_price_sqrt, next_tick_index) 
        = compute_swap_tick(tick_index_temp, config.tick_space, price_sqrt_temp, tick_data.total_liquidity, &offer_token, remain, config.fee_rate);

      // update
      remain = remain - offer_amount_; 
      total_return_amount = total_return_amount + return_amount;
      total_commission_amount = total_commission_amount + commission_amount;
      tick_index_temp = next_tick_index;
      price_sqrt_temp = next_price_sqrt;
    }

    Ok(SimulationResponse {
      return_amount: total_return_amount - total_commission_amount,
      commission_amount: total_commission_amount
    })
  }

  fn swap_simulation_reverse(&self, deps: Deps, asset: Asset) -> StdResult<ReverseSimulationResponse> {
    let price_sqrt = self.current_price_sqrt.load(deps.storage)?;
    let tick_index = self.current_tick_index.load(deps.storage)?;
    let config = self.config.load(deps.storage)?;

    let mut remain = asset.amount.clone();
    let mut tick_index_temp = tick_index.clone();
    let mut price_sqrt_temp = price_sqrt.clone();
    let mut total_offer_amount = Uint128::zero();
    let mut total_commission_amount = Uint128::zero();

    let return_token: TokenNumber;

    let asset_infos: [AssetInfo; 2] = config.asset_infos;

    if asset.info.equal(&asset_infos[0]) {
      return_token = TokenNumber::Token0;
    } else if asset.info.equal(&asset_infos[1]){
      return_token = TokenNumber::Token1;
    } else {
      return Err(StdError::generic_err("Token missmatched"));
    }
    
    while remain > Uint128::zero() {
      let tick_data = match self.tick_data.may_load(deps.storage, NewInt32Key::from(tick_index_temp))? {
        Some(tick_data) => tick_data,
        None => return Err(StdError::generic_err("Can't swap"))
      };

      // compute swap
      let (offer_amount_, return_amount, commission_amount, next_price_sqrt, next_tick_index) 
        = compute_swap_tick_reverse(tick_index_temp, config.tick_space, price_sqrt_temp, tick_data.total_liquidity, &return_token, remain, config.fee_rate);

      // update
      remain = remain + commission_amount - return_amount; 
      total_offer_amount = total_offer_amount + offer_amount_;
      total_commission_amount = total_commission_amount + commission_amount;
      tick_index_temp = next_tick_index;
      price_sqrt_temp = next_price_sqrt;
    }

    Ok(ReverseSimulationResponse {
      offer_amount: total_offer_amount,
      commission_amount: total_commission_amount
    })
  }

  pub fn get_liquidity_info(&self, querier: QuerierWrapper, lp_contract: String, token_id: String) -> StdResult<LiquidityInfoResponse> {
    Ok(querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
      contract_addr: lp_contract,
      msg: to_binary(&LiquidityInfo{ token_id })?,
    }))?)
  }
}

impl<'a> PairContract<'a> {
  pub fn query(&self, deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
      QueryMsg::PairInfo {} => to_binary(&self.pair_info(deps)?),
      QueryMsg::TickInfo { tick_index } => to_binary(&self.tick_info(deps, tick_index)?), 
      QueryMsg::TickInfos { start_after, limit } => {
        to_binary(&self.tick_infos(deps, start_after, limit)?)
      },
      QueryMsg::ProvideCalculation { asset, upper_tick_index, lower_tick_index } 
        => to_binary(&self.provide_calculation(deps, asset, upper_tick_index, lower_tick_index)?),
      QueryMsg::WithdrawCalculation { token_id } 
        => to_binary(&self.withdraw_calculation(deps, token_id)?),
      QueryMsg::Simulation { offer_asset } => to_binary(&self.swap_simulation(deps, offer_asset)?),
      QueryMsg::ReverseSimulation { ask_asset } => to_binary(&self.swap_simulation_reverse(deps, ask_asset)?),
      QueryMsg::CumulativeVolume {} => to_binary(&self.cumulative_volume.load(deps.storage)?),
    }
  }
}
