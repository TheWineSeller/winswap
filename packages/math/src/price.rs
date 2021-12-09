use cosmwasm_std::{Decimal256, Uint256, Uint128};
use wineswap::asset::TokenNumber;
use crate::u256::uints::{U512, U256};
use crate::u256::{div};
use crate::tick::DENOMINATOR;

pub fn compute_price(
  price_sqrt: Uint256, // Q128.128
  liquidity: Uint128,
  amount: Uint128,
  offer_token: &TokenNumber
) -> Uint256 {
  // conver to Q128.128
  let liquidity: U256 = U256::from(liquidity);
  let amount: U256 = U256::from(amount);
  let price_sqrt: U256 = U256::from(price_sqrt);
  
  match offer_token {
    // buy
    TokenNumber::Token1 => {
      (price_sqrt + div(amount << 128, liquidity, false)).into()
    }
    // sell
    TokenNumber::Token0 => {
      let liquidity = liquidity << 128;
      div(liquidity, (liquidity / price_sqrt) + amount, true).into()
    }
  }
}

#[test]
fn price_test() {
  let price = compute_price(
    DENOMINATOR,
    //10000000000, 10000000000
    Uint128::from(10000000000u128),
    Uint128::from(10000000000u128), 
    // sell
    &TokenNumber::Token0
  );

  // price 0.25 price_sqrt 0.5
  assert_eq!(Decimal256::from_ratio(1u128, 2u128), Decimal256::from_ratio(price, DENOMINATOR));

  let price = compute_price(
    DENOMINATOR,
    //10000000000, 10000000000
    Uint128::from(10000000000u128),
    Uint128::from(10000000000u128), 
    // but
    &TokenNumber::Token1
  );

  // price 4 price_sqrt 2
  assert_eq!(Decimal256::from_ratio(2u128, 1u128), Decimal256::from_ratio(price, DENOMINATOR));
}

pub fn price_sqrt_to_price(
  price_sqrt: Uint256, // Q128.128
) -> Decimal256 {
  let price_sqrt: U512 = U256::from(price_sqrt).into();
  let price = price_sqrt * price_sqrt;
  let price = price >> 128;
  Decimal256::from_ratio(
    U256::from(price),
    DENOMINATOR
  )
}