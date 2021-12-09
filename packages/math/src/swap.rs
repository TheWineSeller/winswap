use cosmwasm_std::{Decimal, Uint128, Uint256};
use wineswap::asset::TokenNumber;

use crate::u256::uints::U256;
use crate::u256::{mul_div, div};
use crate::tick::{get_tick_price_sqrt, DENOMINATOR};
use crate::liquidity::get_token_amount_from_liquidity;
use crate::price::{compute_price};

pub fn compute_swap_tick(
  tick_index: i32,
  tick_space: u16,
  current_price_sqrt: Uint256,
  liquidity: Uint128,
  offer_token: &TokenNumber,
  offer_amount: Uint128,
  fee_rate: Decimal
) -> (Uint128, Uint128, Uint128, Uint256, i32) {
  let tick_space_i32 = i32::from(tick_space);
  let price_low_sqrt = get_tick_price_sqrt(tick_index * tick_space_i32);
  let price_high_sqrt = get_tick_price_sqrt((tick_index + 1) * tick_space_i32);

  // out of price range or 0 liquidity
  if current_price_sqrt > price_high_sqrt || current_price_sqrt < price_low_sqrt || liquidity == Uint128::zero() {
    // return dummpy 
    return (Uint128::zero(), Uint128::zero(), Uint128::zero(), Uint256::zero(), 0)
  }

  let (token0_amount, token1_amount) = get_token_amount_from_liquidity(tick_index, tick_index, tick_space, current_price_sqrt, liquidity);
  let (token0_amount_max, _) = get_token_amount_from_liquidity(tick_index, tick_index, tick_space, price_low_sqrt, liquidity);
  let (_, token1_amount_max) = get_token_amount_from_liquidity(tick_index, tick_index, tick_space, price_high_sqrt, liquidity);

  match offer_token {
    // sell
    TokenNumber::Token0 => {
      // tick is exactly same as tick's floor price
      if current_price_sqrt == price_low_sqrt {
        return (
          Uint128::zero(),
          Uint128::zero(),
          Uint128::zero(),
          price_low_sqrt,
          tick_index - 1
        )
      }

      // max offer amount from current tick
      let max_offer_amount = token0_amount_max - token0_amount;

      if offer_amount > max_offer_amount {
        return (
          max_offer_amount,
          token1_amount,
          token1_amount * fee_rate,
          price_low_sqrt,
          tick_index - 1
        )
      } else {
        let next_price_sqrt = compute_price(current_price_sqrt, liquidity, offer_amount, offer_token);
        let return_amount = compute_swap(offer_token, current_price_sqrt, next_price_sqrt, liquidity);
        return (
          offer_amount,
          return_amount,
          return_amount * fee_rate,
          next_price_sqrt,
          tick_index
        )
      }
    },
    TokenNumber::Token1 => {
      // tick is exactly same as tick's ceiling price
      if current_price_sqrt == price_high_sqrt {
        return (
          Uint128::zero(),
          Uint128::zero(),
          Uint128::zero(),
          price_high_sqrt,
          tick_index + 1
        )
      }

      // max offer amount from current tick
      let max_offer_amount = token1_amount_max - token1_amount;

      if offer_amount > max_offer_amount {
        return (
          max_offer_amount,
          token0_amount,
          token0_amount * fee_rate,
          price_high_sqrt,
          tick_index + 1
        )
      } else {
        let next_price_sqrt = compute_price(current_price_sqrt, liquidity, offer_amount, offer_token);
        let return_amount = compute_swap(offer_token, current_price_sqrt, next_price_sqrt, liquidity);
        return (
          offer_amount,
          return_amount,
          return_amount * fee_rate,
          next_price_sqrt,
          tick_index
        )
      }
    },
  }
}

// compute return amount, have to rounding down,
fn compute_swap(
  offer_token: &TokenNumber,
  price_sqrt_before: Uint256,
  price_sqrt_after: Uint256,
  liquidity: Uint128
) -> Uint128 {
  let liquidity_q128: U256 = U256::from(liquidity) << 128;
  let price_sqrt_before: U256 = price_sqrt_before.into();
  let price_sqrt_after: U256 = price_sqrt_after.into();
  match offer_token {
    // sell
    TokenNumber::Token0 => {
      mul_div(liquidity.into(), price_sqrt_before - price_sqrt_after, U256::from(DENOMINATOR) , false).into()
    }
    // buy
    TokenNumber::Token1 => {
      div(
        mul_div(liquidity_q128, price_sqrt_after - price_sqrt_before, price_sqrt_after, false),
        price_sqrt_before, false
      ).into()
    }
  }
}

#[test]
fn swap_test() {
  // buy
  let (_, return_amount, _, _, _) = compute_swap_tick(
    0,
    20000u16,
    DENOMINATOR,
    Uint128::from(10000000000u128),
    &TokenNumber::Token1,
    Uint128::from(10000000000u128),
    Decimal::zero()
  );

  let theorical_amount = Uint128::from(5000000000u128);
  let diff = if theorical_amount > return_amount {
    theorical_amount - return_amount
  } else {
    return_amount - theorical_amount
  };
  
  // can be diff == 1 
  assert!(diff <= Uint128::from(1u128));

  // sell  
  let (_, return_amount, _, _, _) = compute_swap_tick(
    -1,
    20000u16,
    DENOMINATOR,
    Uint128::from(10000000000u128),
    &TokenNumber::Token0,
    Uint128::from(10000000000u128),
    Decimal::zero()
  );

  let theorical_amount = Uint128::from(5000000000u128);
  let diff = if theorical_amount > return_amount {
    theorical_amount - return_amount
  } else {
    return_amount - theorical_amount
  };
  
  // can be diff == 1 
  assert!(diff <= Uint128::from(1u128))
}


/// compute swap in one tick
/// return (offer amount, return amount, commission amount, sqrt of next price, next tick)
pub fn compute_swap_tick_reverse(
  tick_index: i32,
  tick_space: u16,
  current_price_sqrt: Uint256,
  liquidity: Uint128,
  return_token: &TokenNumber,
  return_amount_: Uint128,
  fee_rate: Decimal
) -> (Uint128, Uint128, Uint128, Uint256, i32) {
  let tick_space_i32 = i32::from(tick_space);
  let price_low_sqrt = get_tick_price_sqrt(tick_index * tick_space_i32);
  let price_high_sqrt = get_tick_price_sqrt((tick_index + 1) * tick_space_i32);

  // out of price range or 0 liquidity
  if current_price_sqrt > price_high_sqrt || current_price_sqrt < price_low_sqrt || liquidity == Uint128::zero() {
    // dummy 
    return (Uint128::zero(), Uint128::zero(), Uint128::zero(), Uint256::zero(), 0)
  }

  let one_minus_fee = Decimal::one() - fee_rate;
  let decimal_denominator = Uint128::from(1_000_000_000_000_000_000u128);
  let inv_one_minus_fee = Decimal::from_ratio(decimal_denominator, one_minus_fee * decimal_denominator);

  // amount that have to actually return 
  let return_amount = return_amount_ * inv_one_minus_fee;

  let (token0_amount, token1_amount) = get_token_amount_from_liquidity(tick_index, tick_index, tick_space, current_price_sqrt, liquidity);
  let (token0_amount_max, _) = get_token_amount_from_liquidity(tick_index, tick_index, tick_space, price_low_sqrt, liquidity);
  let (_, token1_amount_max) = get_token_amount_from_liquidity(tick_index, tick_index, tick_space, price_high_sqrt, liquidity);

  match return_token {
    // sell, offer token is Token0  
    TokenNumber::Token1 => {
      if current_price_sqrt == price_low_sqrt {
        return (
          Uint128::zero(),
          Uint128::zero(),
          Uint128::zero(),
          price_low_sqrt,
          tick_index - 1
        )
      }
      
      // max offer amount from current tick
      let max_offer_amount = token0_amount_max - token0_amount;

      if return_amount > token1_amount {
        return (
          max_offer_amount,
          token1_amount,
          token1_amount * fee_rate,
          price_low_sqrt,
          tick_index - 1
        )
      } else {
        // last computation, next_price_sqrt doesn't needed 
        let offer_amount = compute_swap_reverse(return_token, current_price_sqrt, return_amount.into(), liquidity);
        return (
          offer_amount,
          return_amount,
          return_amount * fee_rate,
          current_price_sqrt,
          tick_index
        )
      }
    },
    TokenNumber::Token0 => {
      // tick is exactly same as tick's ceiling price
      if current_price_sqrt == price_high_sqrt {
        return (
          Uint128::zero(),
          Uint128::zero(),
          Uint128::zero(),
          price_high_sqrt,
          tick_index + 1
        )
      }

      // max offer amount from current tick
      let max_offer_amount = token1_amount_max - token1_amount;

      if return_amount > token0_amount {
        return (
          max_offer_amount,
          token0_amount,
          token0_amount * fee_rate,
          price_high_sqrt,
          tick_index + 1
        )
      } else {
        // last computation, next_price_sqrt doesn't needed 
        let offer_amount = compute_swap_reverse(return_token, current_price_sqrt, return_amount.into(), liquidity);
        return (
          offer_amount,
          return_amount,
          return_amount * fee_rate,
          current_price_sqrt,
          tick_index
        )
      }
    },
  }
}

// compute offer amount
fn compute_swap_reverse(
  return_token: &TokenNumber,
  price_sqrt_before: Uint256,
  ask_amount: Uint256,
  liquidity: Uint128
) -> Uint128 {
  let liquidity_q128: U256 = U256::from(liquidity) << 128;
  let price_sqrt_before: U256 = price_sqrt_before.into();
  let ask_amount: U256 = ask_amount.into();
  match return_token {
    // buy
    // offer_amount = (liquidity * price_sprt * ask_amount)/(liquidity/price_sqrt - ask_amount)
    TokenNumber::Token0 => {
      mul_div(
        // liquidity * ask_amount / (liquidity/price_sqrt - ask_amount)
        mul_div(liquidity.into(), ask_amount, div(liquidity_q128, price_sqrt_before, true) - ask_amount, true),
        price_sqrt_before, DENOMINATOR.into(), true
      ).into()
    }
    // sell
    // offer_amount = (liquidity * ask_amount / price_sqrt) / (liquidity * price_sqrt - ask_amount)
    TokenNumber::Token1 => {
      div(
        mul_div(ask_amount << 128, liquidity.into(), price_sqrt_before, true),
        mul_div(liquidity.into(), price_sqrt_before, DENOMINATOR.into(), true) - ask_amount, true
      ).into()
    }
  }
}

#[test]
fn swap_reverse_test() {
  // buy
  let (offer_amount, _, _, _, _) = compute_swap_tick_reverse(
    0,
    30000u16,
    DENOMINATOR * Uint256::from(2u128),
    Uint128::from(20000000000u128),
    &TokenNumber::Token0,
    Uint128::from(5000000000u128),
    Decimal::zero()
  );

  let theorical_amount = Uint128::from(40000000000u128);
  let diff = if theorical_amount > offer_amount {
    theorical_amount - offer_amount
  } else {
    offer_amount - theorical_amount
  };
  
  // can be diff == 1 
  assert!(diff <= Uint128::from(1u128));

  // sell
  let (offer_amount, _, _, _, _) = compute_swap_tick_reverse(
    -1,
    30000u16,
    DENOMINATOR,
    Uint128::from(10000000000u128),
    &TokenNumber::Token1,
    Uint128::from(5000000000u128),
    Decimal::zero()
  );

  let theorical_amount = Uint128::from(10000000000u128);
  let diff = if theorical_amount > offer_amount {
    theorical_amount - offer_amount
  } else {
    offer_amount - theorical_amount
  };
  // can be diff == 1 
  assert!(diff <= Uint128::from(1u128))
}