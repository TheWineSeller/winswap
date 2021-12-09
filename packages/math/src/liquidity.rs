use cosmwasm_std::{Uint128, Uint256};
use wineswap::asset::TokenNumber;
use crate::u256::uints::U256;
use crate::u256::{div, mul_div};
use crate::tick::{get_tick_price_sqrt, DENOMINATOR};

// about the rounding up/down.
// the goal: prevent withdraw more amount than provide amount.
// provide process: calculate liquidity from given amount (1) -> recalculate token amount from the liquidity (2)
// withdraw process: calculate token amount from liquidity that was calculated when it provided (3).
// full process: (1, rounding down) -> (2, rounding up) -> (3, rounding down)
// so get_token_amount_from_liquidity - rounding up, compute_liquidity - rounding down

pub fn get_token_amount_from_liquidity(
  upper_tick_index: i32,
  lower_tick_index: i32,
  tick_space: u16,
  current_price_sqrt: Uint256,
  liquidity: Uint128,
) -> (Uint128, Uint128) {
  if upper_tick_index < lower_tick_index {
    return (Uint128::zero(), Uint128::zero())
  }

  // convert to Q128.128
  let liquidity_q128: U256 = U256::from(liquidity) << 128;
  let tick_space_i32 = i32::from(tick_space);
  let price_low_sqrt: U256 = get_tick_price_sqrt(lower_tick_index * tick_space_i32).into();
  let price_high_sqrt: U256 = get_tick_price_sqrt(upper_tick_index * tick_space_i32 + tick_space_i32).into();
  let current_price_sqrt: U256 = current_price_sqrt.into();
  // case1 out of price range (loewer, token1 amount = 0)
  if current_price_sqrt < price_low_sqrt {
    return (
      div(
        mul_div(liquidity_q128, price_high_sqrt - price_low_sqrt, price_high_sqrt, true),
        price_low_sqrt, true
      ).into(),
      Uint128::zero()
    )
  // case2 out of price range (higher, token0 amount = 0)
  } else if current_price_sqrt >= price_high_sqrt{
    return (
      Uint128::zero(),
      mul_div(liquidity.into(), price_high_sqrt - price_low_sqrt, U256::from(DENOMINATOR) , true).into()
    )
  // case3 price in the price range
  } else {
    return (
      div(
        mul_div(liquidity_q128, price_high_sqrt - current_price_sqrt, price_high_sqrt, true),
        current_price_sqrt, true
      ).into(),
      mul_div(liquidity.into(), current_price_sqrt - price_low_sqrt, U256::from(DENOMINATOR) , true).into()
    )
  }
}

#[test]
fn get_amount_from_liquidity_test() {
  // price_low = 0.92774696514123 / price_low.sqrt = 0.963196223591657
  // price_high = 1.077880109096094 / price_high.sqrt = 1.038210050565922
  let upper_tick_index = 9i32;
  let lower_tick_index = -10i32;
  let tick_space = 75u16;
  let liquidity = Uint128::from(100000000000u128);

  // case1. price in price range
  // price = 1
  let current_price_sqrt = DENOMINATOR;

  let (amount0, amount1) = get_token_amount_from_liquidity(
    upper_tick_index,
    lower_tick_index,
    tick_space,
    current_price_sqrt,
    liquidity,
  );

  // theorical amount
  // amount0 = liquidity * (price_high.sqrt - current_price.sqrt) / (price_high.sqrt * current_price.sqrt)
  // = 100000000000 * (1.038210050565922 - 1) / (1.038210050565922 * 1) = 3680377640.83423515765794
  // amount1 = liquidity * (current_price.sqrt - price_low.sqrt)
  // = 100000000000 * (1 - 0.963196223591657) = 3680377640.8343

  assert_eq!(amount0, Uint128::from(3680377641u128));
  assert_eq!(amount1, Uint128::from(3680377641u128));

  // case2. price is over the price ragne
  // price = 4
  let current_price_sqrt = DENOMINATOR * Uint256::from(2u128);

  let (amount0, amount1) = get_token_amount_from_liquidity(
    upper_tick_index,
    lower_tick_index,
    tick_space,
    current_price_sqrt,
    liquidity,
  );

  // theorical amount
  // amount0 = 0
  // amount1 = liquidity * (price_hight.sqrt - price_low.sqrt)
  // = 100000000000 * (1.038210050565922 - 0.963196223591657) = 7501382697.4265

  assert_eq!(amount0, Uint128::from(0u128));
  assert_eq!(amount1, Uint128::from(7501382697u128));

  // case3. price is under the price ragne
  // price = 0.5
  let current_price_sqrt = DENOMINATOR / Uint256::from(2u128);

  let (amount0, amount1) = get_token_amount_from_liquidity(
    upper_tick_index,
    lower_tick_index,
    tick_space,
    current_price_sqrt,
    liquidity,
  );

  // theorical amount
  // amount0 = liquidity * (price_high.sqrt - price_low.sqrt) / (price_high.sqrt * price_low.sqrt)
  // = 100000000000 * (1.038210050565922 - 0.963196223591657) / (1.038210050565922 * 0.963196223591657) = 7501382697.4265
  // amount1 = 0

  assert_eq!(amount0, Uint128::from(7501382697u128));
  assert_eq!(amount1, Uint128::from(0u128));
}


pub fn compute_liquidity(
  token0_amount: Uint128,
  token1_amount: Uint128,
  current_price_sqrt: Uint256,
  upper_tick_index: i32,
  lower_tick_index: i32,
  tick_space: u16
) -> Uint128 {
  if upper_tick_index < lower_tick_index {
    return Uint128::zero()
  }

  let tick_space_i32 = i32::from(tick_space);
  let price_low_sqrt: U256 = get_tick_price_sqrt(lower_tick_index * tick_space_i32).into();
  let price_high_sqrt: U256 = get_tick_price_sqrt(upper_tick_index * tick_space_i32 + tick_space_i32).into();
  let current_price_sqrt: U256 = current_price_sqrt.into();

  // case1 out of price range (lower, token1 amount = 0), use token0 amount
  if current_price_sqrt < price_low_sqrt {  
    U256::mul_shr(
      mul_div(price_high_sqrt, price_low_sqrt, price_high_sqrt - price_low_sqrt, false),
      U256::from(token0_amount),
      128
    ).into()
  // case2 out of price range (higher, token0 amount = 0), use token1 amount
  } else if current_price_sqrt >= price_high_sqrt{
    div(U256::from(token1_amount) * U256::from(DENOMINATOR),  price_high_sqrt - price_low_sqrt, false).into()
  // case3 price in the price range
  } else {
    let liquidity0 = U256::mul_shr(
      mul_div(price_high_sqrt, current_price_sqrt, price_high_sqrt - current_price_sqrt, false),
      U256::from(token0_amount),
      128
    );

    let liquidity1 = div(U256::from(token1_amount) * U256::from(DENOMINATOR),  current_price_sqrt - price_low_sqrt, false).into();

    // return the smaller one
    if liquidity0 > liquidity1 {
      liquidity1.into()
    } else {
      liquidity0.into()
    }
  }
}

#[test]
fn compute_liquidity_test() {
  let token0_amount = Uint128::from(100000000000u128);
  let token1_amount = Uint128::from(100000000000u128);
  // price_low = 0.92774696514123 / price_low.sqrt = 0.963196223591657
  // price_high = 1.077880109096094 / price_high.sqrt = 1.038210050565922
  let upper_tick_index = 9;
  let lower_tick_index = -10;
  let tick_space = 75u16;

  // case1. price in price range
  // price = 1
  let current_price_sqrt = DENOMINATOR;

  let liquidity = compute_liquidity(
    token0_amount,
    token1_amount,
    current_price_sqrt,
    upper_tick_index,
    lower_tick_index,
    tick_space,
  );

  // theorical liquidity
  // liquidity_from_token0 = amount0 * (price_high.sqrt * current_price.sqrt) / (price_high.sqrt - current_price.sqrt)
  // = 100000000000 * (1.038210050565922 * 1) / (1.038210050565922 - 1) = 2717112474831.058164505356612
  // liquidity_from_token1 = liquidity / (current_price.sqrt - price_low.sqrt)
  // = 100000000000 / (1 - 0.963196223591657) = 2717112474831.01029334818155
  // min(liquidity_from_token0, liquidity_from_token1) = 2717112474831.01029334818155

  assert_eq!(liquidity, Uint128::from(2717112474831u128));

  // case2. price is under the price ragne
  // price = 4
  let current_price_sqrt = DENOMINATOR * Uint256::from(2u128);

  let liquidity = compute_liquidity(
    token0_amount,
    token1_amount,
    current_price_sqrt,
    upper_tick_index,
    lower_tick_index,
    tick_space,
  );

  // theorical liquidity
  // liquidity_from_token0 = 0
  // liquidity_from_token1 = liquidity / (price_high.sqrt - price_low.sqrt)
  // = 100000000000 / (1.038210050565922 - 0.963196223591657) = 1333087565767.135284339208911

  assert_eq!(liquidity, Uint128::from(1333087565767u128));

  // case3. price is under the price ragne 
  // price = 0.5
  let current_price_sqrt = DENOMINATOR / Uint256::from(2u128);

  let liquidity = compute_liquidity(
    token0_amount,
    token1_amount,
    current_price_sqrt,
    upper_tick_index,
    lower_tick_index,
    tick_space,
  );

  // theorical liquidity
  // liquidity_from_token0 = amount0 * (price_high.sqrt * price_low.sqrt) / (price_high.sqrt - price_low.sqrt)
  // = 100000000000 * (1.038210050565922 * 0.963196223591657) / (1.038210050565922 - 0.963196223591657) = 1333087565767.135284339208911
  // liquidity_from_token1 = 0

  assert_eq!(liquidity, Uint128::from(1333087565767u128));
}


// get liqudity of one token
// use same algo with compute_liquidity, do not need test.
pub fn compute_token_liquidity(
  token_number: TokenNumber,
  token_amount: Uint128,
  price_high_sqrt: Uint256,
  price_low_sqrt: Uint256,
) -> Uint128 {
  let price_high_sqrt: U256 = price_high_sqrt.into();
  let price_low_sqrt: U256 = price_low_sqrt.into();
  match token_number {
    TokenNumber::Token0 => {
      (
        U256::mul_shr(
          mul_div(price_high_sqrt, price_low_sqrt, price_high_sqrt - price_low_sqrt, false),
          U256::from(token_amount),
          128
        )
      ).into()
    }

    TokenNumber::Token1 => {
      div(U256::from(token_amount) * U256::from(DENOMINATOR),  price_high_sqrt - price_low_sqrt, false).into()
    }
  }
}

// random provide test
// check given amount >= provide_amount

// #[cfg(test)]
// use rand::Rng;
// #[test]
// fn reversable_test() {
//   let mut rng = rand::thread_rng();
//   let tick_space = 75u16;

//   let mut max_diff = Uint128::zero();
//   for _ in 0..1000000 {
//     // random gen
//     let token0_amount = Uint128::from(rng.gen::<u64>());
//     let token1_amount = Uint128::from(rng.gen::<u64>());
//     let upper_tick_index = rng.gen_range(-1000..1000);
//     let lower_tick_index = upper_tick_index - rng.gen_range(0..500);
//     let current_price_sqrt = get_tick_price_sqrt(rng.gen_range(-80000..80000)) 
//       + Uint256::from(Uint128::from(rng.gen_range(0..100000) as u128));

//     // get liquidity from given amount
//     let liquidity = compute_liquidity(
//       token0_amount,
//       token1_amount,
//       current_price_sqrt,
//       upper_tick_index,
//       lower_tick_index,
//       tick_space,
//     );

//     // recalculate liquidity from liquiidty
//     let (amount0, amount1) = get_token_amount_from_liquidity(
//       upper_tick_index,
//       lower_tick_index,
//       tick_space,
//       current_price_sqrt,
//       liquidity,
//     );

//     if max_diff < (token0_amount - amount0).min(token1_amount - amount1) {
//       max_diff = (token0_amount - amount0).min(token1_amount - amount1);
//     }
//     assert!(amount0 <= token0_amount);
//     assert!(amount1 <= token1_amount);
//   }
//   println!("{}", max_diff);
// }