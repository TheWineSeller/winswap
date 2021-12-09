use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Index, IndexList, IndexedMap, Map, MultiIndex, Item};
use wineswap::asset::AssetInfo;
use wineswap::factory::{Config, PairInfo, PairType};


pub struct FactoryContract<'a> {
  pub config: Item<'a, Config>,
  pub temp_pair_info: Item<'a, TmpPairInfo>,
  pub pairs: IndexedMap<'a, Vec<u8>, PairInfo, PairIndexes<'a>>,
  pub pair_type: Map<'a, Vec<u8>, PairType>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TmpPairInfo {
    pub pair_key: Vec<u8>,
    pub asset_infos: [AssetInfo; 2],
    pub pair_type: PairType,
}

pub fn pair_key(asset_infos: &[AssetInfo; 2], pair_type: String) -> Vec<u8> {
  let mut asset_infos = asset_infos.to_vec();
  asset_infos.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

  [asset_infos[0].as_bytes(), asset_infos[1].as_bytes(), pair_type.as_bytes()].concat()
}

pub fn asset_infos_key(asset_infos: &[AssetInfo; 2]) -> Vec<u8> {
  let mut asset_infos = asset_infos.to_vec();
  asset_infos.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

  [asset_infos[0].as_bytes(), asset_infos[1].as_bytes()].concat()
}

impl Default for FactoryContract<'static> {
  fn default() -> Self {
    Self::new(
      "config",
      "temp_pair_info",
      "asset_infos",
      "type_name",
      "pair_type"
    )
  }
}

impl<'a> FactoryContract<'a> {
  fn new(
    config_key: &'a str,
    temp_pair_info_key: &'a str,
    pair_key: &'a str,
    asset_infos_key: &'a str,
    pair_type_key: &'a str,
  ) -> Self {
    let indexes = PairIndexes {
      asset_infos: MultiIndex::new(asset_infos_idx, pair_key, asset_infos_key),
    };
    Self {
      config: Item::new(config_key),
      temp_pair_info: Item::new(temp_pair_info_key),
      pairs: IndexedMap::new(pair_key, indexes),
      pair_type: Map::new(pair_type_key)
    }
  }
}

pub struct PairIndexes<'a> {
  pub asset_infos: MultiIndex<'a, (Vec<u8>, Vec<u8>), PairInfo>,
}


impl<'a> IndexList<PairInfo> for PairIndexes<'a> {
  fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<PairInfo>> + '_> {
    let v: Vec<&dyn Index<PairInfo>> = vec![&self.asset_infos];
    Box::new(v.into_iter())
  }
}

pub fn asset_infos_idx(d: &PairInfo, k: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
  let asset_infos_key = asset_infos_key(&d.asset_infos);
  (asset_infos_key, k)
}