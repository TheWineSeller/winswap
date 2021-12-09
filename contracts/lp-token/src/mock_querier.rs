use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Coin, ContractResult, CustomQuery, Decimal, Decimal256, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, WasmQuery,
};
use std::collections::HashMap;

use wineswap::pair::{TickInfosResponse, PairInfoResponse, TickInfoResponse, TickInfo};
use wineswap::asset::{AssetInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EmptyCustomQuery {}

// implement custom query
impl CustomQuery for EmptyCustomQuery {}

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    PairInfo {},
    TickInfos { start_after: Option<i32>, limit: Option<u32> }
}


pub struct WasmMockQuerier {
    base: MockQuerier<EmptyCustomQuery>,
    fee_info_querier: FeeInfoQuerier,
    pair_info_querier: PairInfoQuerier,
}

#[derive(Clone, Default)]
pub struct FeeInfoQuerier {
    // this lets us iterate over all pairs that match the first string
    fee_infos: HashMap<i32, TickInfo>,
}

impl FeeInfoQuerier {
    pub fn new(fee_infos: &[(&i32, &TickInfo)]) -> Self {
        let mut temp: HashMap<i32, TickInfo> = HashMap::new();
        for (tick, info) in fee_infos.iter() {
            let info_ = info.clone().clone();
            temp.insert(**tick, info_);
        }
      
        FeeInfoQuerier {
            fee_infos: temp
        }
    }
}

#[derive(Clone, Default)]
pub struct PairInfoQuerier {
  // <pair_addr , asset_infos>
  pairs: HashMap<String, [AssetInfo; 2]>
}

impl PairInfoQuerier {
    pub fn new(pair_infos: &[(&String, &[AssetInfo; 2])]) -> Self {
        let mut temp: HashMap<String, [AssetInfo; 2]> = HashMap::new();
        for (pair, info) in pair_infos.iter() {
            let info_ = info.clone().clone();
            let pair_ = pair.clone().clone();
            temp.insert(pair_, info_);
        }
      
        PairInfoQuerier {
            pairs: temp
        }
    }
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<EmptyCustomQuery> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<EmptyCustomQuery>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                match from_binary(msg) {
                    Ok(QueryMsg::TickInfos { start_after, limit }) => {
                        let mut result = vec![];
                        let limit = if let Some(limit) = limit {
                            limit
                        } else {
                            10
                        };
                        // have to put tick_info strictly
                        if let Some(start_after) = start_after {
                            for tick_index in (start_after+1) .. (start_after+limit as i32+1) {
                                let tick_info = self.fee_info_querier.fee_infos.get(&tick_index);
                                if let Some(tick_info) = tick_info {
                                    let tick_info_ = tick_info.clone();
                                    result.push(TickInfoResponse{
                                        tick_index,
                                        tick_info: tick_info_
                                    })
                                }
                            }
                        };
                        SystemResult::Ok(ContractResult::Ok(
                            to_binary(&TickInfosResponse {
                                infos: result,
                            })
                            .unwrap(),
                        ))
                    }
                    Ok(QueryMsg::PairInfo {}) => {
                        let asset_infos = self.pair_info_querier.pairs.get(contract_addr);
                        if let Some(asset_infos) = asset_infos {
                            let asset_infos_ = asset_infos.clone();
                            SystemResult::Ok(ContractResult::Ok(
                                to_binary(&PairInfoResponse{
                                    liquidity_token: "liquidity".to_string(),
                                    asset_infos: asset_infos_,
                                    tick_space: 20,
                                    fee_rate: Decimal::zero(),
                                    price: Decimal256::one(),
                                    current_tick_index: 0
                                })
                                .unwrap(),
                            ))
                        } else {
                            panic!("No pair")
                        }
                    }
                    _ => panic!("DO NOT ENTER HERE"),
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<EmptyCustomQuery>) -> Self {
        WasmMockQuerier {
            base,
            fee_info_querier: FeeInfoQuerier::default(),
            pair_info_querier: PairInfoQuerier::default()
        }
    }

    pub fn with_fee_infos(&mut self, fee_infos: &[(&i32, &TickInfo)]) {
        self.fee_info_querier = FeeInfoQuerier::new(fee_infos);
    }

    pub fn with_pair_info(&mut self, pair: &[(&String, &[AssetInfo; 2])]) {
        self.pair_info_querier = PairInfoQuerier::new(pair)
    }
}