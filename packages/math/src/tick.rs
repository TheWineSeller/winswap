use crate::u256::uints::U256;
use cosmwasm_std::Uint256;

#[cfg(test)]
use cosmwasm_std::Decimal256;

const MULTIPLIERS: [u128; 19] = [
  0xfff97272373d413259a46990580e213a,
  0xfff2e50f5f656932ef12357cf3c7fdcc,
  0xffe5caca7e10e4e61c3624eaa0941cd0,
  0xffcb9843d60f6159c9db58835c926644,
  0xff973b41fa98c081472e6896dfb254c0,
  0xff2ea16466c96a3843ec78b326b52861,
  0xfe5dee046a99a2a811c461f1969c3053,
  0xfcbe86c7900a88aedcffc83b479aa3a4,
  0xf987a7253ac413176f2b074cf7815e54,
  0xf3392b0822b70005940c7a398e4b70f3,
  0xe7159475a2c29b7443b29c7fa6e889d9,
  0xd097f3bdfd2022b8845ad8f792aa5825,
  0xa9f746462d870fdf8a65dc1f90e061e5,
  0x70d869a156d2a1b890bb3df62baf32f7,
  0x31be135f97d08fd981231505542fcfa6,
  0x9aa508b5b7a84e1c677de54f3e99bc9,
  0x5d6af8dedb81196699c329225ee604,
  0x2216e584f5fa1ea926041bedfe98,
  0x48a170391f7dc42444e8fa2
];

// 0x100000000000000000000000000000000
pub const DENOMINATOR: Uint256 = Uint256::new([
  0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 
  0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 
  0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
  0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
]);

pub const MAX_TICK: i32 = 887271i32;
pub const MIN_TICK: i32 = -887272i32;

#[test]
fn denominator_test() {
  assert_eq!(
    DENOMINATOR,
    (U256::from(0x80000000000000000000000000000000u128) + U256::from(0x80000000000000000000000000000000u128)).into()
  );
}

pub fn get_tick_price_sqrt(tick: i32) -> Uint256 {
  let abs_tick: U256 = if tick < 0 {
    U256::from(-tick) 
  } else {
    U256::from(tick)
  };

  let mut ratio = if (abs_tick & U256::from(0x1)) != U256::from(0) {
    U256::from(0xfffcb933bd6fad37aa2d162d1a594001u128)
  } else {
    U256::from(DENOMINATOR)
  };

  let mut bit_and_with = 0x2;

  for multiplier in MULTIPLIERS {
    if (abs_tick & U256::from(bit_and_with)) != U256::from(0) {
      ratio = U256::mul_shr(ratio, U256::from(multiplier), 128);
    }
    bit_and_with = bit_and_with * 2;
  }

  if tick > 0 {
    ratio = U256::MAX / ratio
  } 

  // return price.sqrt() as Q128.128
  ratio.into()
}

#[test]
fn zero_tick() {
  let zero_tick_price = get_tick_price_sqrt(0i32);
  assert_eq!(Decimal256::one(), Decimal256::from_ratio(zero_tick_price, DENOMINATOR))
}

#[test]
fn tick_bigger_than_zero() {
  // some random tick
  let tick = 268419i32;
  let calculator_result = Decimal256::from_ratio(673524059466525288069u128, 1000000000000000u128);
  let tick_price_sqrt = get_tick_price_sqrt(tick);
  let tick_price_sqrt_dec = Decimal256::from_ratio(tick_price_sqrt, DENOMINATOR);
  let diff = if tick_price_sqrt_dec > calculator_result {
    tick_price_sqrt_dec - calculator_result
  } else {
    calculator_result - tick_price_sqrt_dec
  };
  assert!(diff <= Decimal256::from_ratio(5u128, 10000000000000000u128))
}

#[test]
fn tick_smaller_than_zero() {
  // some random tick
  let tick = -23421i32;
  let calculator_result = Decimal256::from_ratio(310059380017268u128, 1000000000000000u128);
  let tick_price_sqrt = get_tick_price_sqrt(tick);
  let tick_price_sqrt_dec = Decimal256::from_ratio(tick_price_sqrt, DENOMINATOR);
  let diff = if tick_price_sqrt_dec > calculator_result {
    tick_price_sqrt_dec - calculator_result
  } else {
    calculator_result - tick_price_sqrt_dec
  };
  assert!(diff <= Decimal256::from_ratio(5u128, 10000000000000000u128));
}

// like binary tree search
pub fn get_tick_from_price_sqrt(price_sqrt: Uint256) -> i32 {
  let mut tick = 0;
  for i in 0..20 {
    let tick_price_sqrt = get_tick_price_sqrt(tick);
    if tick < MIN_TICK {
      tick = tick + (1 << (19 - i));
    } else if tick > MAX_TICK {
      tick = tick - (1 << (19 - i));
    } else if price_sqrt == tick_price_sqrt {
      break;
    } else if price_sqrt > tick_price_sqrt {
      tick = tick + (1 << (19 - i));
      // println!("{}", tick);
      if i == 19 {
        let tick_price_sqrt = get_tick_price_sqrt(tick);
        if price_sqrt < tick_price_sqrt {
          tick = tick - 1
        }
      }
    } else {
      tick = tick - (1 << (19 - i));
      // println!("{}", tick);
      if i == 19 {
        let tick_price_sqrt = get_tick_price_sqrt(tick);
        if price_sqrt < tick_price_sqrt {
          tick = tick - 1
        }
      }
    }
  }

  return tick
}

#[test]
fn tick_from_price_test() {
  // some ramdom tick
  let tick_ = 128947i32;

  let price_sqrt = get_tick_price_sqrt(tick_);

  // when price is exactly same
  let tick = get_tick_from_price_sqrt(price_sqrt);
  assert_eq!(tick, tick_);

  // price btw tick price
  let tick = get_tick_from_price_sqrt(price_sqrt + Uint256::from(100u128));
  assert_eq!(tick, tick_);


  let tick_ = -42314i32;

  let price_sqrt = get_tick_price_sqrt(tick_);

  // when price is exactly same
  let tick = get_tick_from_price_sqrt(price_sqrt);
  assert_eq!(tick, tick_);

  // price btw tick price
  let tick = get_tick_from_price_sqrt(price_sqrt + Uint256::from(100u128));
  assert_eq!(tick, tick_);

  let tick_ = 0i32;

  let price_sqrt = get_tick_price_sqrt(tick_);

  // when price is exactly same
  let tick = get_tick_from_price_sqrt(price_sqrt);
  assert_eq!(tick, tick_);

  // price btw tick price
  let tick = get_tick_from_price_sqrt(price_sqrt + Uint256::from(100u128));
  assert_eq!(tick, tick_);


  // // for all valid tick
  // for i in -887272..887272 {
  //   let price_sqrt = get_tick_price_sqrt(i);

  //   // when price is exactly same
  //   let tick = get_tick_from_price_sqrt(price_sqrt);
  //   assert_eq!(tick, i);

  //   // price btw tick price
  //   let tick = get_tick_from_price_sqrt(price_sqrt + Uint256::from(100u128));
  //   assert_eq!(tick, i);

  //   if i % 100 == 0 {
  //     println!("{}", i)
  //   }
  // }
}

pub fn tick_to_tick_index(tick: i32, tick_space: u16) -> i32 {
  if tick >= 0 {
    tick / i32::from(tick_space)
  } else if tick % i32::from(tick_space) == 0 {
    tick / i32::from(tick_space)
  } else {
    tick / i32::from(tick_space) - 1i32
  }
}

#[test]
fn tick_to_tick_index_test() {
  let tick_index = tick_to_tick_index(301i32, 30u16);
  assert_eq!(tick_index, 10i32);

  let tick_index = tick_to_tick_index(-19i32, 30u16);
  assert_eq!(tick_index, -1i32);

  let tick_index = tick_to_tick_index(-60i32, 30u16);
  assert_eq!(tick_index, -2i32);

  let tick_index = tick_to_tick_index(0i32, 3124u16);
  assert_eq!(tick_index, 0i32);
}

pub fn tick_index_to_tick(tick: i32, tick_space: u16) -> i32 {
  tick * i32::from(tick_space)
}