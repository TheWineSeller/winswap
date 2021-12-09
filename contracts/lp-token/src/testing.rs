use cosmwasm_std::{from_binary, to_binary, Addr, Decimal, Uint128, SubMsg, StdError, CosmosMsg, WasmMsg, Timestamp};
use cosmwasm_std::testing::{mock_env, mock_info};

use wineswap::lp_token::{InstantiateMsg, QueryMsg, ExecuteMsg, ConfigResponse, Approval, LpReceiveMsg};
use wineswap::pair::{TickInfo, ExecuteMsg as PairExecuteMsg};
use wineswap::asset::{AssetInfo, Asset};
use cw0::Expiration;


use crate::state::{LpContract, LiquidityInfo, FeeInfo};
use crate::mock_querier::mock_dependencies;
use crate::error::ContractError;

#[test]
fn instantiate_test(){
  let lp_token = LpContract::default();
  // instantiate test
  let mut deps = mock_dependencies(&[]);

  let instantiate_msg = InstantiateMsg {
    name: "wine_lp".to_string(),
    symbol: "WINELP".to_string(),
    minter: "pair".to_string(),
  };

  let info = mock_info("pair", &[]);

  let _res = lp_token.instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

  let query_res = lp_token.query(deps.as_ref(), QueryMsg::Config {}).unwrap();
  let config_res: ConfigResponse = from_binary(&query_res).unwrap();

  assert_eq!("wine_lp".to_string(), config_res.name);
  assert_eq!("WINELP".to_string(), config_res.symbol);
  assert_eq!("pair".to_string(), config_res.minter);
}

#[test]
fn mint_burn_test() {
  let lp_token = LpContract::default();

  let mut deps = mock_dependencies(&[]);

  let instantiate_msg = InstantiateMsg {
    name: "wine_lp".to_string(),
    symbol: "WINELP".to_string(),
    minter: "pair".to_string(),
  };

  let info = mock_info("pair", &[]);

  let _res = lp_token.instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

  deps.querier.with_fee_infos(&[
    (&1, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&2, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&3, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&4, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&5, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&6, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&7, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&8, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&9, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&10, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
  ]);

  let mint_mgs = ExecuteMsg::Mint {
    liquidity: Uint128::from(10u128),
    upper_tick_index: 7,
    lower_tick_index: 2,
    owner: "owner".to_string()
  };
  let env =  mock_env();
  let _res = lp_token.execute(deps.as_mut(), env.clone(), info.clone(), mint_mgs.clone()).unwrap();
  let token = lp_token.tokens.load(deps.as_mut().storage, "0").unwrap();

  assert_eq!(
    LiquidityInfo {
      owner: Addr::unchecked("owner"),
      liquidity: Uint128::from(10u128),
      upper_tick_index: 7,
      lower_tick_index: 2,
      last_updated_fee_infos: vec![
        FeeInfo{ tick_index: 2, last_fee_growth_0: Decimal::one(), last_fee_growth_1: Decimal::one() },
        FeeInfo{ tick_index: 3, last_fee_growth_0: Decimal::one(), last_fee_growth_1: Decimal::one() },
        FeeInfo{ tick_index: 4, last_fee_growth_0: Decimal::one(), last_fee_growth_1: Decimal::one() },
        FeeInfo{ tick_index: 5, last_fee_growth_0: Decimal::one(), last_fee_growth_1: Decimal::one() },
        FeeInfo{ tick_index: 6, last_fee_growth_0: Decimal::one(), last_fee_growth_1: Decimal::one() },
        FeeInfo{ tick_index: 7, last_fee_growth_0: Decimal::one(), last_fee_growth_1: Decimal::one() },
      ],
      approvals: vec![]
    },
    token
  );

  // try to mint who is not pair
  let info_not_pair = mock_info("not_pair", &[]);
  let res = lp_token.execute(deps.as_mut(), env.clone(), info_not_pair.clone(), mint_mgs);

  match res {
    Err(ContractError::Unauthorized {}) => assert!(true),
    _ => panic!("Must return unauthorized error"),
  }

  let burn_msgs = ExecuteMsg::Burn {
    token_id: "0".to_string()
  };

  // try to burn who is not pair
  let res = lp_token.execute(deps.as_mut(), env.clone(), info_not_pair, burn_msgs.clone());
  match res {
    Err(ContractError::Unauthorized {}) => assert!(true),
    _ => panic!("Must return unauthorized error"),
  }

  // checked token removed
  let _res = lp_token.execute(deps.as_mut(), env.clone(), info, burn_msgs.clone());
  let token = lp_token.tokens.load(deps.as_mut().storage, "0");
  match token {
    Err(StdError::NotFound { kind: _ }) => assert!(true),
    _ => panic!("Must return error"),
  }
}

#[test]
fn transfer_send_test() {
  // instantiate
  let lp_token = LpContract::default();

  let mut deps = mock_dependencies(&[]);

  let instantiate_msg = InstantiateMsg {
    name: "wine_lp".to_string(),
    symbol: "WINELP".to_string(),
    minter: "pair".to_string(),
  };

  let info = mock_info("pair", &[]);

  let _res = lp_token.instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

  deps.querier.with_fee_infos(&[
    (&1, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&2, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&3, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&4, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&5, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&6, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&7, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&8, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&9, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&10, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
  ]);

  // mint one
  let mint_mgs = ExecuteMsg::Mint {
    liquidity: Uint128::from(10u128),
    upper_tick_index: 7,
    lower_tick_index: 2,
    owner: "owner".to_string()
  };
  let env =  mock_env();
  let _res = lp_token.execute(deps.as_mut(), env.clone(), info, mint_mgs.clone()).unwrap();

  // transfer token
  let transfer_msg = ExecuteMsg::Transfer{
    recipient: "next_owner".to_string(),
    token_id: "0".to_string()
  };

  let info = mock_info("owner", &[]);

  let _res = lp_token.execute(deps.as_mut(), env.clone(), info, transfer_msg.clone()).unwrap();

  let token = lp_token.tokens.load(deps.as_mut().storage, "0").unwrap();
  // owner changed
  assert_eq!(token.owner, Addr::unchecked("next_owner"));

  // try transfer who is not owner
  let info = mock_info("not_owner", &[]);

  let res = lp_token.execute(deps.as_mut(), env.clone(), info, transfer_msg.clone());

  match res {
    Err(ContractError::Unauthorized{}) => assert!(true),
    _ => panic!("Must return unauthorized error"),
  }

  let info = mock_info("next_owner", &[]);

  let send_msg = ExecuteMsg::Send {
    contract: "market".to_string(),
    token_id: "0".to_string(),
    msg: to_binary("some").unwrap()
  };

  let receive_msg = LpReceiveMsg {
    sender: info.sender.to_string(),
    token_id: "0".to_string(),
    msg: to_binary("some").unwrap()
  };

  let res = lp_token.execute(deps.as_mut(), env.clone(), info, send_msg).unwrap();

  assert_eq!(
    res.messages,
    vec![
      SubMsg::new(receive_msg.into_cosmos_msg("market".to_string()).unwrap()),
    ]
  )
}


#[test]
fn approve_revoke_test() {
  // instantiate
  let lp_token = LpContract::default();

  let mut deps = mock_dependencies(&[]);

  let instantiate_msg = InstantiateMsg {
    name: "wine_lp".to_string(),
    symbol: "WINELP".to_string(),
    minter: "pair".to_string(),
  };

  let info = mock_info("pair", &[]);

  let _res = lp_token.instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

  deps.querier.with_fee_infos(&[
    (&1, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&2, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&3, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&4, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&5, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&6, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&7, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&8, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&9, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&10, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
  ]);

  // mint one
  let mint_mgs = ExecuteMsg::Mint {
    liquidity: Uint128::from(10u128),
    upper_tick_index: 7,
    lower_tick_index: 2,
    owner: "owner".to_string()
  };
  let env =  mock_env();
  let _res = lp_token.execute(deps.as_mut(), env.clone(), info, mint_mgs.clone()).unwrap();

  // approve never
  let approve_msg = ExecuteMsg::Approve {
    spender: "market".to_string(),
    expires: None,
    token_id: "0".to_string()
  };

  let owner_info = mock_info("owner", &[]);

  let _res = lp_token.execute(deps.as_mut(), env.clone(), owner_info.clone(), approve_msg.clone()).unwrap();
  let token = lp_token.tokens.load(deps.as_mut().storage, "0").unwrap();
  assert_eq!(token.approvals, vec![Approval{spender: Addr::unchecked("market"), expires: Expiration::Never{}}]);

  // reapprove to same spender
  let approve_msg = ExecuteMsg::Approve {
    spender: "market".to_string(),
    expires: Some(Expiration::AtHeight(100000)),
    token_id: "0".to_string()
  };

  let owner_info = mock_info("owner", &[]);

  let _res = lp_token.execute(deps.as_mut(), env.clone(), owner_info.clone(), approve_msg.clone()).unwrap();
  let token = lp_token.tokens.load(deps.as_mut().storage, "0").unwrap();
  assert_eq!(token.approvals, vec![Approval{spender: Addr::unchecked("market"), expires: Expiration::AtHeight(100000)}]);

  // try approve already expired to same spender
  // * mock_env's height = 12345
  let approve_msg = ExecuteMsg::Approve {
    spender: "market".to_string(),
    expires: Some(Expiration::AtHeight(10000)),
    token_id: "0".to_string()
  };

  let owner_info = mock_info("owner", &[]);
  
  let res = lp_token.execute(deps.as_mut(), env.clone(), owner_info.clone(), approve_msg.clone());
  
  match res {
    Err(ContractError::Expired {}) => assert!(true),
    _ => panic!("Must return expired error"),
  }

  // try apporve who is not the owner
  let not_owner_info = mock_info("not_owner", &[]);
  let approve_msg = ExecuteMsg::Approve {
    spender: "market".to_string(),
    expires: None,
    token_id: "0".to_string()
  };

  let res = lp_token.execute(deps.as_mut(), env.clone(), not_owner_info.clone(), approve_msg.clone());

  match res {
    Err(ContractError::Unauthorized{}) => assert!(true),
    _ => panic!("Must return unauthorized error"),
  }

  // revoke
  let revoek_msg = ExecuteMsg::Revoke {
    spender: "market".to_string(),
    token_id: "0".to_string()
  };

  let _res = lp_token.execute(deps.as_mut(), env.clone(), owner_info.clone(), revoek_msg.clone()).unwrap();

  let token = lp_token.tokens.load(deps.as_mut().storage, "0").unwrap();
  // empty approvals
  assert_eq!(token.approvals, vec![]);

  // try apporve who is not the owner
  let not_owner_info = mock_info("not_owner", &[]);
  let approve_msg = ExecuteMsg::Approve {
    spender: "market".to_string(),
    expires: None,
    token_id: "0".to_string()
  };

  let _res = lp_token.execute(deps.as_mut(), env.clone(), not_owner_info.clone(), approve_msg.clone());

  // transfer by spender test
  let approve_msg = ExecuteMsg::Approve {
    spender: "market".to_string(),
    expires: None,
    token_id: "0".to_string()
  };

  let owner_info = mock_info("owner", &[]);

  let _res = lp_token.execute(deps.as_mut(), env.clone(), owner_info.clone(), approve_msg.clone()).unwrap();

  let transfer_msg = ExecuteMsg::Transfer {
    recipient: "next_owner".to_string(),
    token_id: "0".to_string()
  };

  let market_info = mock_info("market", &[]);

  let _res = lp_token.execute(deps.as_mut(), env.clone(), market_info, transfer_msg.clone()).unwrap();

  let token = lp_token.tokens.load(deps.as_mut().storage, "0").unwrap();
  // transfered
  assert_eq!(token.owner, "next_owner");
  // reset approvals
  assert_eq!(token.approvals, vec![]);
}

#[test]
fn claim_reward_test() {
  // instantiate
  let lp_token = LpContract::default();

  let mut deps = mock_dependencies(&[]);

  let instantiate_msg = InstantiateMsg {
    name: "wine_lp".to_string(),
    symbol: "WINELP".to_string(),
    minter: "pair".to_string(),
  };

  let info = mock_info("pair", &[]);

  let _res = lp_token.instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

  deps.querier.with_fee_infos(&[
    (&1, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&2, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&3, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&4, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&5, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&6, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&7, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&8, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&9, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&10, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
  ]);

  // mint one
  let mint_mgs = ExecuteMsg::Mint {
    liquidity: Uint128::from(10u128),
    upper_tick_index: 7,
    lower_tick_index: 2,
    owner: "owner".to_string()
  };
  let env =  mock_env();
  let _res = lp_token.execute(deps.as_mut(), env.clone(), info, mint_mgs.clone()).unwrap();

  // fee_growth changed
  deps.querier.with_fee_infos(&[
    (&1, &TickInfo {
      last_fee_growth_0: Decimal::from_ratio(2u128, 1u128),
      last_fee_growth_1: Decimal::from_ratio(2u128, 1u128),
      total_liquidity: Uint128::from(10u128),
    }),
    (&2, &TickInfo {
      last_fee_growth_0: Decimal::from_ratio(2u128, 1u128),
      last_fee_growth_1: Decimal::from_ratio(2u128, 1u128),
      total_liquidity: Uint128::from(10u128),
    }),
    (&3, &TickInfo {
      last_fee_growth_0: Decimal::from_ratio(2u128, 1u128),
      last_fee_growth_1: Decimal::from_ratio(2u128, 1u128),
      total_liquidity: Uint128::from(10u128),
    }),
    (&4, &TickInfo {
      last_fee_growth_0: Decimal::from_ratio(2u128, 1u128),
      last_fee_growth_1: Decimal::from_ratio(2u128, 1u128),
      total_liquidity: Uint128::from(10u128),
    }),
    (&5, &TickInfo {
      last_fee_growth_0: Decimal::from_ratio(2u128, 1u128),
      last_fee_growth_1: Decimal::from_ratio(2u128, 1u128),
      total_liquidity: Uint128::from(10u128),
    }),
    (&6, &TickInfo {
      last_fee_growth_0: Decimal::from_ratio(2u128, 1u128),
      last_fee_growth_1: Decimal::from_ratio(2u128, 1u128),
      total_liquidity: Uint128::from(10u128),
    }),
    (&7, &TickInfo {
      last_fee_growth_0: Decimal::from_ratio(2u128, 1u128),
      last_fee_growth_1: Decimal::from_ratio(2u128, 1u128),
      total_liquidity: Uint128::from(10u128),
    }),
    (&8, &TickInfo {
      last_fee_growth_0: Decimal::from_ratio(2u128, 1u128),
      last_fee_growth_1: Decimal::from_ratio(2u128, 1u128),
      total_liquidity: Uint128::from(10u128),
    }),
    (&9, &TickInfo {
      last_fee_growth_0: Decimal::from_ratio(2u128, 1u128),
      last_fee_growth_1: Decimal::from_ratio(2u128, 1u128),
      total_liquidity: Uint128::from(10u128),
    }),
    (&10, &TickInfo {
      last_fee_growth_0: Decimal::from_ratio(2u128, 1u128),
      last_fee_growth_1: Decimal::from_ratio(2u128, 1u128),
      total_liquidity: Uint128::from(10u128),
    }),
  ]);

  deps.querier.with_pair_info(&[
    (&"pair".to_string(), &[AssetInfo::Token {contract_addr: "wine".to_string()}, AssetInfo::NativeToken {denom: "uusd".to_string()}])
  ]);

  let update_liquidity_msg = ExecuteMsg::ClaimReward{
    token_id: "0".to_string()
  };

  // try execute by who is not owner
  let info = mock_info("not_owner", &[]);

  let res = lp_token.execute(deps.as_mut(), env.clone(), info.clone(), update_liquidity_msg.clone());

  match res {
    Err(ContractError::Unauthorized {}) => assert!(true),
    _ => panic!("Must return unauthorized error"),
  }

  let info = mock_info("owner", &[]);

  let mut env_next = mock_env();
  // add 1y 
  env_next.block.time = Timestamp::from_seconds(env.block.time.seconds() + 31570560u64);

  let res = lp_token.execute(deps.as_mut(), env_next.clone(), info.clone(), update_liquidity_msg.clone()).unwrap();

  let rewards = [
    Asset {
      info: AssetInfo::Token {contract_addr: "wine".to_string()},
      amount: Uint128::from(60u128)
    },
    Asset {
      info: AssetInfo::NativeToken {denom: "uusd".to_string()},
      amount: Uint128::from(60u128)
    }
  ];

  assert_eq!(
    res.messages,
    vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
      contract_addr: "pair".to_string(),
      msg: to_binary(&PairExecuteMsg::ClaimReward {
        token_id: "0".to_string(),
        rewards: rewards,
      }).unwrap(),
      funds: vec![],
    }))]
  );

  let token = lp_token.tokens.load(deps.as_mut().storage, "0").unwrap();

  assert_eq!(
    LiquidityInfo {
      owner: Addr::unchecked("owner"),
      liquidity: Uint128::from(10u128),
      upper_tick_index: 7,
      lower_tick_index: 2,
      last_updated_fee_infos: vec![
        FeeInfo{ tick_index: 2, last_fee_growth_0: Decimal::from_ratio(2u128, 1u128), last_fee_growth_1: Decimal::from_ratio(2u128, 1u128) },
        FeeInfo{ tick_index: 3, last_fee_growth_0: Decimal::from_ratio(2u128, 1u128), last_fee_growth_1: Decimal::from_ratio(2u128, 1u128) },
        FeeInfo{ tick_index: 4, last_fee_growth_0: Decimal::from_ratio(2u128, 1u128), last_fee_growth_1: Decimal::from_ratio(2u128, 1u128) },
        FeeInfo{ tick_index: 5, last_fee_growth_0: Decimal::from_ratio(2u128, 1u128), last_fee_growth_1: Decimal::from_ratio(2u128, 1u128) },
        FeeInfo{ tick_index: 6, last_fee_growth_0: Decimal::from_ratio(2u128, 1u128), last_fee_growth_1: Decimal::from_ratio(2u128, 1u128) },
        FeeInfo{ tick_index: 7, last_fee_growth_0: Decimal::from_ratio(2u128, 1u128), last_fee_growth_1: Decimal::from_ratio(2u128, 1u128) },
      ],
      approvals: vec![]
    },
    token
  );

  // test for pair
  let info = mock_info("pair", &[]);

  let _res = lp_token.execute(deps.as_mut(), env_next.clone(), info.clone(), update_liquidity_msg).unwrap();
}

#[test]
fn update_liquidity_test() {
  // instantiate
  let lp_token = LpContract::default();

  let mut deps = mock_dependencies(&[]);

  let instantiate_msg = InstantiateMsg {
    name: "wine_lp".to_string(),
    symbol: "WINELP".to_string(),
    minter: "pair".to_string(),
  };

  let info = mock_info("pair", &[]);

  let _res = lp_token.instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

  deps.querier.with_fee_infos(&[
    (&1, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&2, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&3, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&4, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&5, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&6, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&7, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&8, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&9, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
    (&10, &TickInfo {
      last_fee_growth_0: Decimal::one(),
      last_fee_growth_1: Decimal::one(),
      total_liquidity: Uint128::from(10u128),
    }),
  ]);

  // mint one
  let mint_mgs = ExecuteMsg::Mint {
    liquidity: Uint128::from(10u128),
    upper_tick_index: 7,
    lower_tick_index: 2,
    owner: "owner".to_string()
  };
  let env =  mock_env();
  let _res = lp_token.execute(deps.as_mut(), env.clone(), info, mint_mgs.clone()).unwrap();

  // try to update by who is not pair(minter)

  let info = mock_info("not_pair", &[]);

  let update_liquidity_msg = ExecuteMsg::UpdateLiquidity {
    amount: Uint128::from(10u128),
    token_id: "0".to_string(),
    add: true
  };

  let res = lp_token.execute(deps.as_mut(), env.clone(), info, update_liquidity_msg.clone());

  match res {
    Err(ContractError::Unauthorized {}) => assert!(true),
    _ => panic!("Must return unauthorized error"),
  }

  // update (add)
  let info = mock_info("pair", &[]);

  let update_liquidity_msg = ExecuteMsg::UpdateLiquidity {
    amount: Uint128::from(10u128),
    token_id: "0".to_string(),
    add: true
  };

  let _res = lp_token.execute(deps.as_mut(), env.clone(), info.clone(), update_liquidity_msg.clone()).unwrap();

  let token = lp_token.tokens.load(&deps.storage, "0").unwrap();

  assert_eq!(Uint128::from(20u128), token.liquidity);

  // update (withdraw)

  let update_liquidity_msg = ExecuteMsg::UpdateLiquidity {
    amount: Uint128::from(10u128),
    token_id: "0".to_string(),
    add: false
  };

  let _res = lp_token.execute(deps.as_mut(), env.clone(), info.clone(), update_liquidity_msg.clone()).unwrap();

  let token = lp_token.tokens.load(&deps.storage, "0").unwrap();

  assert_eq!(Uint128::from(10u128), token.liquidity)
}