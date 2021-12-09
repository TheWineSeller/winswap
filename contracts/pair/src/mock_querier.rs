use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Coin, ContractResult, CustomQuery, Decimal, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, WasmQuery, Uint128
};
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};
use std::collections::HashMap;

use wineswap::lp_token::{RewardResponse, LiquidityInfoResponse};
use wineswap::asset::{Asset, AssetInfo};
use wineswap::factory::Config;

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
    TickInfos { start_after: Option<i32>, limit: Option<u32> },
    LiquidityInfo { token_id: String },
    Reward { token_id: String },
    Config {}
}


pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    tax_querier: TaxQuerier,
    lp_querier: LpQuerier,
}

#[derive(Clone, Default)]
pub struct LpQuerier {
    liquidity_infos: HashMap<String, LiquidityInfoResponse>,
}

impl LpQuerier {
    pub fn new(liquidity_infos: &[(&String, &LiquidityInfoResponse)]) -> Self {
        let mut temp: HashMap<String, LiquidityInfoResponse> = HashMap::new();
        for (token_id, info) in liquidity_infos.iter() {
            let token_id_ = token_id.clone().clone();
            let info_ = info.clone().clone();
            temp.insert(token_id_, info_);
        }
      
        LpQuerier {
            liquidity_infos: temp
        }
    }
}

#[derive(Clone, Default)]
pub struct TaxQuerier {
    rate: Decimal,
    // this lets us iterate over all pairs that match the first string
    caps: HashMap<String, Uint128>,
}

impl TaxQuerier {
    pub fn new(rate: Decimal, caps: &[(&String, &Uint128)]) -> Self {
        TaxQuerier {
            rate,
            caps: caps_to_map(caps),
        }
    }
}

pub(crate) fn caps_to_map(caps: &[(&String, &Uint128)]) -> HashMap<String, Uint128> {
    let mut owner_map: HashMap<String, Uint128> = HashMap::new();
    for (denom, cap) in caps.iter() {
        owner_map.insert(denom.to_string(), **cap);
    }
    owner_map
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
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
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(TerraQueryWrapper { route, query_data }) => {
                if route == &TerraRoute::Treasury {
                    match query_data {
                        TerraQuery::TaxRate {} => {
                            let res = TaxRateResponse {
                                rate: self.tax_querier.rate,
                            };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        TerraQuery::TaxCap { denom } => {
                            let cap = self
                                .tax_querier
                                .caps
                                .get(denom)
                                .copied()
                                .unwrap_or_default();
                            let res = TaxCapResponse { cap };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr: _, msg }) => {
                match from_binary(msg) {
                    Ok(QueryMsg::LiquidityInfo { token_id }) => {
                        SystemResult::Ok(ContractResult::Ok(
                            to_binary(self.lp_querier.liquidity_infos.get(&token_id).unwrap())
                            .unwrap(),
                        ))
                    }
                    Ok(QueryMsg::Reward { token_id:_ }) => {
                        SystemResult::Ok(ContractResult::Ok(
                            // dummy
                            to_binary(&RewardResponse { rewards: [
                                Asset {
                                    info: AssetInfo::Token {contract_addr: "wine".to_string()},
                                    amount: Uint128::from(100u128)
                                },
                                Asset {
                                    info: AssetInfo::NativeToken {denom: "uusd".to_string()},
                                    amount: Uint128::from(100u128)
                                },
                            ]})
                            .unwrap(),
                        ))
                    }
                    Ok(QueryMsg::Config {}) => {
                        SystemResult::Ok(ContractResult::Ok(
                            // factory config
                            to_binary(&Config {
                                owner: Addr::unchecked("owner"),
                                pair_code_id: 123,
                                token_code_id: 312,
                            })
                            .unwrap(),
                        ))
                    }
                    _ =>  panic!("DO NOT ENTER HERE"),
                        
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            tax_querier: TaxQuerier::default(),
            lp_querier: LpQuerier::default()
        }
    }
    pub fn with_lp_infos(&mut self, lp_infos: &[(&String, &LiquidityInfoResponse)]) {
        self.lp_querier = LpQuerier::new(lp_infos);
    }

    // configure the token owner mock querier
    pub fn with_tax(&mut self, rate: Decimal, caps: &[(&String, &Uint128)]) {
        self.tax_querier = TaxQuerier::new(rate, caps);
    }
}