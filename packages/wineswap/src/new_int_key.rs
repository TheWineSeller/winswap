use cw_storage_plus::{Prefixer, PrimaryKey};
use std::convert::TryInto;
use std::marker::PhantomData;

// this auto-implements PrimaryKey for all the IntKey types
impl<'a> PrimaryKey<'a> for NewInt32Key {
  type Prefix = ();
  type SubPrefix = ();

  fn key(&self) -> Vec<&[u8]> {
      self.wrapped.key()
  }
}

// this auto-implements Prefixer for all the IntKey types
impl Prefixer<'_> for NewInt32Key {
  fn prefix(&self) -> Vec<&[u8]> {
      self.wrapped.prefix()
  }
}

/// It will cast one-particular int type into a Key via Vec<u8>, ensuring you don't mix up u32 and u64
/// You can use new or the from/into pair to build a key from an int:
///
///   let k = U64Key::new(12345);
///   let k = U32Key::from(12345);
///   let k: U16Key = 12345.into();
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewInt32Key {
  pub wrapped: Vec<u8>,
  pub data: PhantomData<i32>,
}

impl NewInt32Key {
  pub fn new(val: i32) -> Self {
      NewInt32Key {
          wrapped: wrap_i32(val),
          data: PhantomData,
      }
  }
}

impl From<NewInt32Key> for i32 {
  fn from(val: NewInt32Key) -> Self {
    let mut wrapped: [u8; 4] = vec_to_array(val.wrapped);
    if wrapped[0] >= 128 {
      wrapped[0] = wrapped[0] - 128;
    } else {
      wrapped[0] = wrapped[0] + 128;
    }
    i32::from_be_bytes(wrapped)
  }
}

impl From<i32> for NewInt32Key {
  fn from(val: i32) -> Self {
    NewInt32Key::new(val)
  }
}

impl From<Vec<u8>> for NewInt32Key {
  fn from(wrap: Vec<u8>) -> Self {
      NewInt32Key {
          wrapped: wrap,
          data: PhantomData,
      }
  }
}

impl From<NewInt32Key> for Vec<u8> {
  fn from(k: NewInt32Key) -> Vec<u8> {
    k.wrapped
  }
}

fn wrap_i32(n: i32) -> Vec<u8>{
  let mut byte = n.to_be_bytes();
  if n == 0 {
    byte = [128, 0, 0, 0];
  } else if n > 0 {
    byte[0] = byte[0] + 128;
  } else{
    byte[0] = byte[0] - 128;
  }
  byte.to_vec()
}

// use this function for convert intkey to array
fn vec_to_array<T>(v: Vec<T>) -> [T; 4] {
  v.try_into()
    .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", 4, v.len()))
}