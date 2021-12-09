use std::convert::TryFrom;
use cosmwasm_std::{Uint128, Uint256};

#[allow(clippy::all)]
pub mod uints {
  uint::construct_uint! {
    pub struct U256(4);
  }
  uint::construct_uint! {
    pub struct U512(8);
  }
}

use uints::{U256, U512};

impl U256 {
  pub fn mul_shr(a: U256, b: U256, shift: u16) -> U256 {
		let a = U512::from(a);
    let b = U512::from(b);
    let mul = a * b >> shift;
    mul.into()
	}
}

impl From<U256> for Uint256 {
  fn from(original: U256) -> Self {
    Uint256::try_from(original.to_string().as_str()).unwrap()
  }
}

impl From<U256> for Uint128 {
  fn from(original: U256) -> Self {
    Uint128::try_from(original.to_string().as_str()).unwrap()
  }
}

impl From<Uint256> for U256 {
  fn from(original: Uint256) -> Self {
    U256::from(original.to_be_bytes())
  }
}

impl From<Uint128> for U256 {
  fn from(original: Uint128) -> Self {
    let original = Uint256::from(original);
    U256::from(original.to_be_bytes())
  }
}

impl From<U256> for U512 {
	fn from(value: U256) -> U512 {
		let U256(ref arr) = value;
		let mut ret = [0; 8];
		ret[0] = arr[0];
		ret[1] = arr[1];
		ret[2] = arr[2];
		ret[3] = arr[3];
		U512(ret)
	}
}

impl From<U512> for U256 {
	fn from(value: U512) -> U256 {
		let U512(ref arr) = value;
		if arr[4] | arr[5] | arr[6] | arr[7] != 0 {
			panic!("Overflow");
		}
		let mut ret = [0; 4];
		ret[0] = arr[0];
		ret[1] = arr[1];
		ret[2] = arr[2];
		ret[3] = arr[3];
		U256(ret)
	}
}

// math

pub fn mul_div(a: U256, b: U256, c:U256, rounding_up: bool) -> U256 {
  let a: U512 = a.into();
  let b: U512 = b.into();
  let c: U512 = c.into();

  let mul = a * b;

  let muldiv = div512(mul, c, rounding_up);

  muldiv.into()
}

pub fn div(a: U256, b: U256, rounding_up: bool) -> U256 {
  let mut div = a / b;

  if rounding_up {
    let even = b % 2 == U256::from(0);
    if even && (a % b) >= (b / 2) {
      div = div + 1;
    } else if (a % b) > (b / 2) {
      div = div + 1;
    }
  }

  div
}

pub fn div512(a: U512, b: U512, rounding_up: bool) -> U512 {
  let mut div = a / b;

  if rounding_up {
    let even = b % 2 == U512::from(0);
    if even && (a % b) >= (b / 2) {
      div = div + 1;
    } else if (a % b) > (b / 2) {
      div = div + 1;
    }
  }

  div
}

#[test]
fn div_test() {
  let rounding_up = div(U256::from(3), U256::from(2), true);

  assert_eq!(U256::from(2), rounding_up);

  let rounding_up = div(U256::from(5), U256::from(4), true);

  assert_eq!(U256::from(1), rounding_up);

  let not_rounding_up = div(U256::from(5), U256::from(3), false);

  assert_eq!(U256::from(1), not_rounding_up);

  let not_rounding_up = div(U256::from(4), U256::from(3), false);

  assert_eq!(U256::from(1), not_rounding_up);
}