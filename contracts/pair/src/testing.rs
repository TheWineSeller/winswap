use cosmwasm_std::{to_binary, Addr, Decimal, CosmosMsg, SubMsg, WasmMsg, Uint128, Coin, ReplyOn};
use cosmwasm_std::testing::{mock_env, mock_info};
use wineswap::pair::{InstantiateMsg, ExecuteMsg, Cw20HookMsg, TickIndexes};
use wineswap::lp_token::{InstantiateMsg as TokenInstantiateMsg, ExecuteMsg as TokenExecuteMsg, LiquidityInfoResponse};
use wineswap::asset::{Asset, AssetInfo, TokenNumber};

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use wineswap_math::{
  liquidity::{get_token_amount_from_liquidity, compute_liquidity},
  tick::DENOMINATOR,
  swap::{compute_swap_tick}
};
use crate::state::PairContract;
use crate::mock_querier::mock_dependencies;
use crate::error::ContractError;

#[test]
fn instantiate_test() {
  let pair = PairContract::default();

  let mut deps = mock_dependencies(&[]);

  let instantiate_msg = InstantiateMsg {
    asset_infos: [
      AssetInfo::Token { contract_addr: "wine".to_string() },
      AssetInfo::NativeToken { denom: "uusd".to_string() }
    ],
    fee_rate: Decimal::from_ratio(1u128, 100u128),
    token_code_id: 123,
    tick_space: 100,
    initial_price: Decimal::one()
  };

  let info = mock_info("factory", &[]);
  let env = mock_env();
  let res = pair.instantiate(deps.as_mut(), env.clone(), info, instantiate_msg).unwrap();

  assert_eq!(
    res.messages,
    vec![SubMsg {
      msg: WasmMsg::Instantiate {
        admin: None,
        code_id: 123u64,
        label: "".to_string(),
        funds: vec![],
        msg: to_binary(&TokenInstantiateMsg{
          name: "wineswap concentrated liquidity token".to_string(),
          symbol: ("WINELP".to_string()),
          minter: env.contract.address.to_string(),
        }).unwrap(),
      }
      .into(),
      gas_limit: None,
      id: 1,
      reply_on: ReplyOn::Success,
    }]
  );

  let config = pair.config.load(&deps.storage).unwrap();

  assert_eq!(config.asset_infos, [
    AssetInfo::Token { contract_addr: "wine".to_string() },
    AssetInfo::NativeToken { denom: "uusd".to_string() }
  ]);
  assert_eq!(config.fee_rate, Decimal::from_ratio(1u128, 100u128));
  assert_eq!(config.tick_space, 100);
  // reply doesn't execute so it is still temp addr
  assert_eq!(config.liquidity_token, "factory".to_string());

  let current_tick_index = pair.current_tick_index.load(&deps.storage).unwrap();

  assert_eq!(current_tick_index, 0);

  // 0 tick space
  let instantiate_msg = InstantiateMsg {
    asset_infos: [
      AssetInfo::Token { contract_addr: "wine".to_string() },
      AssetInfo::NativeToken { denom: "uusd".to_string() }
    ],
    fee_rate: Decimal::from_ratio(1u128, 100u128),
    token_code_id: 123,
    tick_space: 0,
    initial_price: Decimal::one()
  };

  let info = mock_info("factory", &[]);
  let env = mock_env();
  let res = pair.instantiate(deps.as_mut(), env.clone(), info, instantiate_msg);

  match res {
    Err(_) => assert!(true),
    _ => panic!("Must return invalid tick space error"),
  }

  // invalid fee rate
  let instantiate_msg = InstantiateMsg {
    asset_infos: [
      AssetInfo::Token { contract_addr: "wine".to_string() },
      AssetInfo::NativeToken { denom: "uusd".to_string() }
    ],
    fee_rate: Decimal::from_ratio(1000u128, 100u128),
    token_code_id: 123,
    tick_space: 0,
    initial_price: Decimal::one()
  };

  let info = mock_info("factory", &[]);
  let env = mock_env();
  let res = pair.instantiate(deps.as_mut(), env.clone(), info, instantiate_msg);

  match res {
    Err(_) => assert!(true),
    _ => panic!("Must return invalid fee rate error"),
  }
}

#[test]
fn provide_withdraw_test() {
  // instantiate
  let pair = PairContract::default();

  let mut deps = mock_dependencies(&[]);

  let instantiate_msg = InstantiateMsg {
    asset_infos: [
      AssetInfo::Token { contract_addr: "wine".to_string() },
      AssetInfo::NativeToken { denom: "uusd".to_string() }
    ],
    fee_rate: Decimal::from_ratio(1u128, 100u128),
    token_code_id: 123,
    tick_space: 100,
    initial_price: Decimal::one()
  };

  let info = mock_info("factory", &[]);
  let env = mock_env();
  let _res = pair.instantiate(deps.as_mut(), env.clone(), info, instantiate_msg).unwrap();

  let mut config = pair.config.load(&deps.storage).unwrap();
  config.liquidity_token = Addr::unchecked("liquidity");
  pair.config.save(deps.as_mut().storage, &config).unwrap();

  deps.querier.with_tax(
    Decimal::zero(),
    &[(&"uusd".to_string(), &Uint128::from(1000000u128))],
  );

  let provide_msg = ExecuteMsg::ProvideLiquidity {
    token_id: None,
    tick_indexes: Some(TickIndexes {
      upper_tick_index: 10,
      lower_tick_index: -10,
    }),
    assets: [
      Asset {
        info: AssetInfo::NativeToken {denom: "uusd".to_string()},
        amount: Uint128::from(1000000u128)
      },
      Asset {
        info: AssetInfo::Token {contract_addr: "wine".to_string()},
        amount: Uint128::from(1000000u128)
      }
    ]
  };

  let info = mock_info("user", &[Coin{ denom: "uusd".to_string(), amount: Uint128::from(1000000u128)}]);

  // calculate amount
  let liquidity = compute_liquidity(Uint128::from(1000000u128), Uint128::from(1000000u128), DENOMINATOR, 10, -10, 100);
  let (amount0, amount1) = get_token_amount_from_liquidity(10, -10, 100, DENOMINATOR, liquidity);

  let res = pair.execute(deps.as_mut(), mock_env(), info.clone(), provide_msg).unwrap();

  deps.querier.with_lp_infos(&[
    (&"0".to_string(), &LiquidityInfoResponse{
      approvals: vec![],
      liquidity,
      upper_tick_index: 10,
      lower_tick_index: -10,
      owner: Addr::unchecked("user")
    })
  ]);
  
  // get return amount
  let return_uusd = Asset {
    info: AssetInfo::NativeToken {denom: "uusd".to_string()},
    amount: Uint128::from(1000000u128) - amount1
  };

  assert_eq!(
    res.messages,
    vec![
      // tranfer from wine
      SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: "wine".to_string(),
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
          owner: "user".to_string(),
          recipient: env.contract.address.to_string(),
          amount: amount0,
        }).unwrap(),
        funds: vec![],
      })),
      // refund uusd
      SubMsg::new(return_uusd.into_msg(&deps.as_mut().querier, info.sender.clone()).unwrap()),
      // mint
      SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: "liquidity".to_string(),
        msg: to_binary(&TokenExecuteMsg::Mint {
          owner: "user".to_string(),
          liquidity,
          upper_tick_index: 10,
          lower_tick_index: -10
        }).unwrap(),
        funds: vec![],
      }))
    ]
  );

  // additional provide
  let provide_msg = ExecuteMsg::ProvideLiquidity {
    token_id: Some("0".to_string()),
    tick_indexes: None,
    assets: [
      Asset {
        info: AssetInfo::NativeToken {denom: "uusd".to_string()},
        amount: Uint128::from(1000000u128)
      },
      Asset {
        info: AssetInfo::Token {contract_addr: "wine".to_string()},
        amount: Uint128::from(1000000u128)
      }
    ]
  };

  let info = mock_info("user", &[Coin{ denom: "uusd".to_string(), amount: Uint128::from(1000000u128)}]);

  // calculate amount
  let liquidity = compute_liquidity(Uint128::from(1000000u128), Uint128::from(1000000u128), DENOMINATOR, 10, -10, 100);
  let (amount0, amount1) = get_token_amount_from_liquidity(10, -10, 100, DENOMINATOR, liquidity);

  let res = pair.execute(deps.as_mut(), mock_env(), info.clone(), provide_msg).unwrap();

  deps.querier.with_lp_infos(&[
    (&"0".to_string(), &LiquidityInfoResponse{
      approvals: vec![],
      liquidity: liquidity * Uint128::from(2u128),
      upper_tick_index: 10,
      lower_tick_index: -10,
      owner: Addr::unchecked("user")
    })
  ]);
  
  // get return amount
  let return_uusd = Asset {
    info: AssetInfo::NativeToken {denom: "uusd".to_string()},
    amount: Uint128::from(1000000u128) - amount1
  };

  assert_eq!(
    res.messages,
    vec![
      // tranfer from wine
      SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: "wine".to_string(),
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
          owner: "user".to_string(),
          recipient: env.contract.address.to_string(),
          amount: amount0,
        }).unwrap(),
        funds: vec![],
      })),
      // refund uusd
      SubMsg::new(return_uusd.into_msg(&deps.as_mut().querier, info.sender.clone()).unwrap()),
      // claim reward
      SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: "liquidity".to_string(),
        msg: to_binary(&TokenExecuteMsg::ClaimReward {
          token_id: "0".to_string()
        }).unwrap(),
        funds: vec![],
      })),
      // update liquidity
      SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: "liquidity".to_string(),
        msg: to_binary(&TokenExecuteMsg::UpdateLiquidity {
          token_id: "0".to_string(),
          amount: liquidity,
          add: true,
        }).unwrap(),
        funds: vec![],
      }))
    ]
  );

  // try additional provide who is not the owner
  let provide_msg = ExecuteMsg::ProvideLiquidity {
    token_id: Some("0".to_string()),
    tick_indexes: None,
    assets: [
      Asset {
        info: AssetInfo::NativeToken {denom: "uusd".to_string()},
        amount: Uint128::from(1000000u128)
      },
      Asset {
        info: AssetInfo::Token {contract_addr: "wine".to_string()},
        amount: Uint128::from(1000000u128)
      }
    ]
  };

  let info = mock_info("not_owner", &[Coin{ denom: "uusd".to_string(), amount: Uint128::from(1000000u128)}]);

  let res = pair.execute(deps.as_mut(), mock_env(), info.clone(), provide_msg);

  match res {
    Err(ContractError::Unauthorized {}) => assert!(true),
    _ => panic!("Must return unauthorized error"),
  }

  // invalid provide (tick)
  let provide_msg = ExecuteMsg::ProvideLiquidity {
    token_id: None,
    tick_indexes: Some(TickIndexes {
      upper_tick_index: -10,
      lower_tick_index: 10,
    }),
    assets: [
      Asset {
        info: AssetInfo::NativeToken {denom: "uusd".to_string()},
        amount: Uint128::from(1000000u128)
      },
      Asset {
        info: AssetInfo::Token {contract_addr: "wine".to_string()},
        amount: Uint128::from(1000000u128)
      }
    ]
  };

  let res = pair.execute(deps.as_mut(), mock_env(), info.clone(), provide_msg);
  match res {
    Err(ContractError::InvalidTickRange {}) => assert!(true),
    _ => panic!("Must return invalid tick error"),
  }

  // invalid provide (zero liquidity)
  let provide_msg = ExecuteMsg::ProvideLiquidity {
    token_id: None,
    tick_indexes: Some(TickIndexes {
      upper_tick_index: 10,
      lower_tick_index: -10,
    }),
    assets: [
      Asset {
        info: AssetInfo::NativeToken {denom: "uusd".to_string()},
        amount: Uint128::from(0u128)
      },
      Asset {
        info: AssetInfo::Token {contract_addr: "wine".to_string()},
        amount: Uint128::from(0u128)
      }
    ]
  };

  let info = mock_info("user", &[]);
  let res = pair.execute(deps.as_mut(), mock_env(), info.clone(), provide_msg);
  match res {
    Err(ContractError::ZeroLiquidity {}) => assert!(true),
    _ => panic!("Must return zero liquidity error"),
  }

  let withdraw_msg = ExecuteMsg::WithdrawLiquidity{
    token_id: "0".to_string(),
    amount: None
  };

  // try withdraw who is not the owner of the liqudity
  let info = mock_info("bad_user", &[]);
  let res = pair.execute(deps.as_mut(), mock_env(), info, withdraw_msg.clone());

  match res {
    Err(ContractError::Unauthorized {}) => assert!(true),
    _ => panic!("Must return unauthorized error"),
  }

  // partial withdraw
  let withdraw_msg = ExecuteMsg::WithdrawLiquidity{
    token_id: "0".to_string(),
    amount: Some(liquidity)
  };

  let info = mock_info("user", &[]);
  let res = pair.execute(deps.as_mut(), mock_env(), info.clone(), withdraw_msg).unwrap();

  let assets = [
    Asset {
      info: AssetInfo::Token {contract_addr: "wine".to_string()},
      amount: amount0
    },
    Asset {
      info: AssetInfo::NativeToken {denom: "uusd".to_string()},
      amount: amount1
    },
  ];

  assert_eq!(
    res.messages,
    vec![
      SubMsg::new(assets[0].clone().into_msg(&deps.as_mut().querier, info.sender.clone()).unwrap()),
      SubMsg::new(assets[1].clone().into_msg(&deps.as_mut().querier, info.sender.clone()).unwrap()),
      // claim reward
      SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.liquidity_token.to_string(),
        msg: to_binary(&TokenExecuteMsg::ClaimReward {
          token_id: "0".to_string(),
        }).unwrap(),
        funds: vec![],
      })),
      // update liquidity
      SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: "liquidity".to_string(),
        msg: to_binary(&TokenExecuteMsg::UpdateLiquidity {
          token_id: "0".to_string(),
          amount: liquidity,
          add: false,
        }).unwrap(),
        funds: vec![],
      }))
    ]
  );

  deps.querier.with_lp_infos(&[
    (&"0".to_string(), &LiquidityInfoResponse{
      approvals: vec![],
      liquidity: liquidity,
      upper_tick_index: 10,
      lower_tick_index: -10,
      owner: Addr::unchecked("user")
    })
  ]);

  let withdraw_msg = ExecuteMsg::WithdrawLiquidity{
    token_id: "0".to_string(),
    amount: None
  };
  let info = mock_info("user", &[]);
  let res = pair.execute(deps.as_mut(), mock_env(), info.clone(), withdraw_msg).unwrap();

  assert_eq!(
    res.messages,
    vec![
      SubMsg::new(assets[0].clone().into_msg(&deps.as_mut().querier, info.sender.clone()).unwrap()),
      SubMsg::new(assets[1].clone().into_msg(&deps.as_mut().querier, info.sender.clone()).unwrap()),
      // claim reward
      SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.liquidity_token.to_string(),
        msg: to_binary(&TokenExecuteMsg::ClaimReward {
          token_id: "0".to_string(),
        }).unwrap(),
        funds: vec![],
      })),
      // burn
      SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: "liquidity".to_string(),
        msg: to_binary(&TokenExecuteMsg::Burn { token_id: "0".to_string()}).unwrap(),
        funds: vec![],
      }))
    ]
  );
}

#[test]
fn swap_test() {
  // instantiate
  let pair = PairContract::default();

  let mut deps = mock_dependencies(&[]);

  let instantiate_msg = InstantiateMsg {
    asset_infos: [
      AssetInfo::Token { contract_addr: "wine".to_string() },
      AssetInfo::NativeToken { denom: "uusd".to_string() }
    ],
    fee_rate: Decimal::from_ratio(1u128, 100u128),
    token_code_id: 123,
    tick_space: 100,
    initial_price: Decimal::one()
  };

  let info = mock_info("factory", &[]);
  let env = mock_env();
  let _res = pair.instantiate(deps.as_mut(), env.clone(), info, instantiate_msg).unwrap();

  let mut config = pair.config.load(&deps.storage).unwrap();
  config.liquidity_token = Addr::unchecked("liquidity");
  pair.config.save(deps.as_mut().storage, &config).unwrap();

  deps.querier.with_tax(
    Decimal::zero(),
    &[(&"uusd".to_string(), &Uint128::from(1000000u128))],
  );

  let provide_msg = ExecuteMsg::ProvideLiquidity {
    token_id: None,
    tick_indexes: Some(TickIndexes {
      upper_tick_index: 10,
      lower_tick_index: -10,
    }),
    assets: [
      Asset {
        info: AssetInfo::NativeToken {denom: "uusd".to_string()},
        amount: Uint128::from(1000000u128)
      },
      Asset {
        info: AssetInfo::Token {contract_addr: "wine".to_string()},
        amount: Uint128::from(1000000u128)
      }
    ]
  };

  let info = mock_info("user", &[Coin{ denom: "uusd".to_string(), amount: Uint128::from(1000000u128)}]);

  // calculate liquidity
  let liquidity = compute_liquidity(Uint128::from(1000000u128), Uint128::from(1000000u128), DENOMINATOR, 10, -10, 100);

  let _res = pair.execute(deps.as_mut(), mock_env(), info.clone(), provide_msg).unwrap();

  // swap test
  let swap_msg = ExecuteMsg::Swap{
    offer_asset: Asset{
      info: AssetInfo::NativeToken { denom: "uusd".to_string() },
      amount: Uint128::from(50000u128)
    },
    belief_price: None,
    max_slippage: None,
    to: None
  };

  let info = mock_info("user", &[Coin{ denom: "uusd".to_string(), amount: Uint128::from(50000u128)}]);

  let res = pair.execute(deps.as_mut(), mock_env(), info.clone(), swap_msg).unwrap();

  let (_, return_amount, commission_amount, _, _) = compute_swap_tick(
    0, 100, DENOMINATOR, liquidity, &TokenNumber::Token1, Uint128::from(50000u128), Decimal::from_ratio(1u128, 100u128)
  );

  let return_asset = Asset {
    info: AssetInfo::Token { contract_addr: "wine".to_string() },
    amount: return_amount - commission_amount
  };

  assert_eq!(
    res.messages,
    vec![
      SubMsg::new(return_asset.clone().into_msg(&deps.as_mut().querier, info.sender.clone()).unwrap()),
    ]
  );

  // swap test2 (opposite direction) and to test

  let swap_msg = ExecuteMsg::Receive(
    Cw20ReceiveMsg {
      sender: "user".to_string(),
      amount: Uint128::from(40000u128),
      msg: to_binary(&Cw20HookMsg::Swap {
        belief_price: None,
        max_slippage: None,
        to: Some("user2".to_string())
      }).unwrap()
    }
  );

  let info = mock_info("wine", &[]);

  let current_price_sqrt = pair.current_price_sqrt.load(&deps.storage).unwrap();

  let res = pair.execute(deps.as_mut(), mock_env(), info.clone(), swap_msg).unwrap();

  let (_, return_amount, commission_amount, _, _) = compute_swap_tick(
    0, 100, current_price_sqrt, liquidity, &TokenNumber::Token0, Uint128::from(40000u128), Decimal::from_ratio(1u128, 100u128)
  );

  let return_asset = Asset {
    info: AssetInfo::NativeToken { denom: "uusd".to_string() },
    amount: return_amount - commission_amount
  };

  assert_eq!(
    res.messages,
    vec![
      SubMsg::new(return_asset.clone().into_msg(&deps.as_mut().querier, Addr::unchecked("user2")).unwrap()),
    ]
  );

  // max_slippage test
  let swap_msg = ExecuteMsg::Swap{
    offer_asset: Asset{
      info: AssetInfo::NativeToken { denom: "uusd".to_string() },
      amount: Uint128::from(50000u128)
    },
    // fail due to belief_price is too low
    belief_price: Some(Decimal::from_ratio(1u128, 10u128)),
    max_slippage: Some(Decimal::from_ratio(1u128, 10u128)),
    to: None
  };

  let info = mock_info("user", &[Coin{ denom: "uusd".to_string(), amount: Uint128::from(50000u128)}]);

  let res = pair.execute(deps.as_mut(), mock_env(), info.clone(), swap_msg);

  match res {
    Err(ContractError::MaxSlippage {}) => assert!(true),
    _ => panic!("Must return max slippage error"),
  }

  let swap_msg = ExecuteMsg::Swap{
    offer_asset: Asset{
      info: AssetInfo::NativeToken { denom: "uusd".to_string() },
      amount: Uint128::from(50000u128)
    },
    belief_price: Some(Decimal::from_ratio(1u128, 1u128)),
    max_slippage: Some(Decimal::from_ratio(1u128, 10u128)),
    to: None
  };

  let info = mock_info("user", &[Coin{ denom: "uusd".to_string(), amount: Uint128::from(50000u128)}]);

  let res = pair.execute(deps.as_mut(), mock_env(), info.clone(), swap_msg);

  // swap executed
  match res {
    Err(_) => assert!(false),
    _ => assert!(true)
  }

  // try to swap more than given
  let swap_msg = ExecuteMsg::Swap{
    offer_asset: Asset{
      info: AssetInfo::NativeToken { denom: "uusd".to_string() },
      amount: Uint128::from(50000u128)
    },
    belief_price: Some(Decimal::from_ratio(1u128, 1u128)),
    max_slippage: Some(Decimal::from_ratio(1u128, 10u128)),
    to: None
  };

  let info = mock_info("user", &[Coin{ denom: "uusd".to_string(), amount: Uint128::from(5000u128)}]);

  let res = pair.execute(deps.as_mut(), mock_env(), info.clone(), swap_msg);

  match res {
    Err(_) => assert!(true),
    _ => panic!("Must return balance mismatch error"),
  }

  // try to swap less than given
  let swap_msg = ExecuteMsg::Swap{
    offer_asset: Asset{
      info: AssetInfo::NativeToken { denom: "uusd".to_string() },
      amount: Uint128::from(50000u128)
    },
    belief_price: Some(Decimal::from_ratio(1u128, 1u128)),
    max_slippage: Some(Decimal::from_ratio(1u128, 10u128)),
    to: None
  };

  let info = mock_info("user", &[Coin{ denom: "uusd".to_string(), amount: Uint128::from(500000u128)}]);

  let res = pair.execute(deps.as_mut(), mock_env(), info.clone(), swap_msg);

  match res {
    Err(_) => assert!(true),
    _ => panic!("Must return balance mismatch error"),
  }
}

// claim test
#[test]
fn claim_test() {
  // instantiate
  let pair = PairContract::default();

  let mut deps = mock_dependencies(&[]);

  let instantiate_msg = InstantiateMsg {
    asset_infos: [
      AssetInfo::Token { contract_addr: "wine".to_string() },
      AssetInfo::NativeToken { denom: "uusd".to_string() }
    ],
    fee_rate: Decimal::from_ratio(1u128, 100u128),
    token_code_id: 123,
    tick_space: 100,
    initial_price: Decimal::one()
  };

  let info = mock_info("factory", &[]);
  let env = mock_env();
  let _res = pair.instantiate(deps.as_mut(), env.clone(), info, instantiate_msg).unwrap();

  let mut config = pair.config.load(&deps.storage).unwrap();
  config.liquidity_token = Addr::unchecked("liquidity");
  pair.config.save(deps.as_mut().storage, &config).unwrap();

  deps.querier.with_tax(
    Decimal::zero(),
    &[(&"uusd".to_string(), &Uint128::from(1000000u128))],
  );

  let provide_msg = ExecuteMsg::ProvideLiquidity {
    token_id: None,
    tick_indexes: Some(TickIndexes {
      upper_tick_index: 10,
      lower_tick_index: -10,
    }),
    assets: [
      Asset {
        info: AssetInfo::NativeToken {denom: "uusd".to_string()},
        amount: Uint128::from(1000000u128)
      },
      Asset {
        info: AssetInfo::Token {contract_addr: "wine".to_string()},
        amount: Uint128::from(1000000u128)
      }
    ]
  };

  let info = mock_info("user", &[Coin{ denom: "uusd".to_string(), amount: Uint128::from(1000000u128)}]);

  // calculate amount
  let liquidity = compute_liquidity(Uint128::from(1000000u128), Uint128::from(1000000u128), DENOMINATOR, 10, -10, 100);

  let _res = pair.execute(deps.as_mut(), mock_env(), info.clone(), provide_msg).unwrap();

  deps.querier.with_lp_infos(&[
    (&"0".to_string(), &LiquidityInfoResponse{
      approvals: vec![],
      liquidity,
      upper_tick_index: 10,
      lower_tick_index: -10,
      owner: Addr::unchecked("user")
    })
  ]);

  let mut config = pair.config.load(&deps.storage).unwrap();
  config.liquidity_token = Addr::unchecked("liquidity");
  pair.config.save(deps.as_mut().storage, &config).unwrap();


  let rewards =  [
    Asset {
      info: AssetInfo::Token {contract_addr: "wine".to_string()},
      amount: Uint128::from(1000000u128)
    },
    Asset {
      info: AssetInfo::NativeToken {denom: "uusd".to_string()},
      amount: Uint128::from(1000000u128)
    },
  ];

  let claim_msg = ExecuteMsg::ClaimReward{
    rewards: rewards.clone(),
    token_id: "0".to_string()
  };

  let info = mock_info(&"liquidity".to_string(), &[]);
  let res = pair.execute(deps.as_mut(), mock_env(), info.clone(), claim_msg.clone()).unwrap();

  assert_eq!{
    res.messages,
    vec![
      SubMsg::new(rewards[0].clone().into_msg(&deps.as_mut().querier, Addr::unchecked("user")).unwrap()),
      SubMsg::new(rewards[1].clone().into_msg(&deps.as_mut().querier, Addr::unchecked("user")).unwrap()),
    ]
  }

  // try to execute who is not liquidity token
  let info = mock_info(&"bad_user".to_string(), &[]);
  let res = pair.execute(deps.as_mut(), mock_env(), info.clone(), claim_msg.clone());

  match res {
    Err(ContractError::Unauthorized {}) => assert!(true),
    _ => panic!("Must return unauthoerized error"),
  }
}
