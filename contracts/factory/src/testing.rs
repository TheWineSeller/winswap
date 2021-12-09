use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{attr, from_binary, to_binary, Addr, Decimal, WasmMsg, SubMsg, ReplyOn};

use wineswap::factory::{InstantiateMsg, ExecuteMsg, QueryMsg,
  Config, PairTypeResponse, PairType, PairInfo
};
use wineswap::asset::AssetInfo;
use wineswap::pair::InstantiateMsg as PairInstantiateMsg;
use crate::state::{pair_key, FactoryContract, TmpPairInfo};
use crate::error::ContractError;

#[test]
fn factory_full_test() {
  let factory = FactoryContract::default();
  // instantiate test
  let mut deps = mock_dependencies(&[]);

  let instantiate_msg = InstantiateMsg {
    owner: "owner".to_string(), 
    pair_code_id: 123u64,
    token_code_id: 32u64,
  };

  let info = mock_info("owner", &[]);

  let _res = factory.instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

  let query_res = factory.query(deps.as_ref(), QueryMsg::Config {}).unwrap();
  let config_res: Config = from_binary(&query_res).unwrap(); 
  assert_eq!("owner".to_string(), config_res.owner);
  assert_eq!(123u64, config_res.pair_code_id);
  assert_eq!(32u64, config_res.token_code_id);

  // update config
  // try to update who is not the owner
  let info_with_notowner = mock_info("notowner", &[]);

  // unauthorized
  let update_msg = ExecuteMsg::UpdateConfig {
    owner: Some("notowner".to_string()),
    token_code_id: None,
    pair_code_id: None,
  };

  let res = factory.execute(deps.as_mut(), mock_env(), info_with_notowner, update_msg);

  match res {
    Err(ContractError::Unauthorized {}) => assert!(true),
    _ => panic!("Must return unauthorized error"),
  }

  // owner change
  let update_msg = ExecuteMsg::UpdateConfig {
    owner: Some("next_owner".to_string()),
    token_code_id: None,
    pair_code_id: None,
  };

  let info = mock_info("owner", &[]);
  let _res = factory.execute(deps.as_mut(), mock_env(), info, update_msg);
  let query_res = factory.query(deps.as_ref(), QueryMsg::Config {}).unwrap();
  let config_res: Config = from_binary(&query_res).unwrap(); 
  assert_eq!("next_owner".to_string(), config_res.owner);
  assert_eq!(123u64, config_res.pair_code_id);
  assert_eq!(32u64, config_res.token_code_id);

  // other change
  let update_msg = ExecuteMsg::UpdateConfig {
    owner: None,
    token_code_id: Some(789u64),
    pair_code_id: Some(987u64),
  };

  let info = mock_info("next_owner", &[]);
  let _res = factory.execute(deps.as_mut(), mock_env(), info, update_msg);
  let query_res = factory.query(deps.as_ref(), QueryMsg::Config {}).unwrap();
  let config_res: Config = from_binary(&query_res).unwrap(); 
  assert_eq!("next_owner".to_string(), config_res.owner);
  assert_eq!(987u64, config_res.pair_code_id);
  assert_eq!(789u64, config_res.token_code_id);


  // add pair type

  // new pair type
  let add_pair_type_msg = ExecuteMsg::AddPairType {
    type_name: "type".to_string(),
    tick_space: 150u16,
    fee_rate: Decimal::from_ratio(3u128, 1000u128)
  };

  let info = mock_info("next_owner", &[]);
  let _res = factory.execute(deps.as_mut(), mock_env(), info, add_pair_type_msg);
  let query_res = factory.query(deps.as_ref(), QueryMsg::PairType {type_name: "type".to_string()}).unwrap();
  let pair_type_res: PairTypeResponse = from_binary(&query_res).unwrap();

  assert_eq!("type".to_string(), pair_type_res.type_name);
  assert_eq!(150u16, pair_type_res.tick_space);
  assert_eq!(Decimal::from_ratio(3u128, 1000u128), pair_type_res.fee_rate);


  // type name already taken
  let add_pair_type_msg = ExecuteMsg::AddPairType {
    type_name: "type".to_string(),
    tick_space: 150u16,
    fee_rate: Decimal::from_ratio(3u128, 1000u128)
  };

  let info = mock_info("next_owner", &[]);
  let res = factory.execute(deps.as_mut(), mock_env(), info, add_pair_type_msg);

  match res {
    Err(ContractError::PairTypeExists {}) => assert!(true),
    _ => panic!("Must return pair type exists rate error"),
  }

  // invalid fee rate
  let add_pair_type_msg = ExecuteMsg::AddPairType {
    type_name: "type2".to_string(),
    tick_space: 150u16,
    fee_rate: Decimal::from_ratio(10u128, 1u128)
  };

  let info = mock_info("next_owner", &[]);
  let res = factory.execute(deps.as_mut(), mock_env(), info, add_pair_type_msg);

  match res {
    Err(ContractError::InvalidFeeRate {}) => assert!(true),
    _ => panic!("Must return fee rate error"),
  }

  // type to add type by who is not the owner
  let add_pair_type_msg = ExecuteMsg::AddPairType {
    type_name: "type3".to_string(),
    tick_space: 150u16,
    fee_rate: Decimal::from_ratio(3u128, 1000u128)
  };

  let info = mock_info("not_owner", &[]);
  let res = factory.execute(deps.as_mut(), mock_env(), info, add_pair_type_msg);

  match res {
    Err(ContractError::Unauthorized {}) => assert!(true),
    _ => panic!("Must return unauthorized error"),
  }

  // create pair

  let asset_infos = [
    AssetInfo::Token {
      contract_addr: "wine".to_string(),
    },
    AssetInfo::NativeToken {
      denom: "uusd".to_string()
    }
  ];

  let create_pair_msg = ExecuteMsg::CreatePair {
    asset_infos: asset_infos.clone(),
    initial_price: Decimal::one(),
    pair_type: "type".to_string()
  };

  let info = mock_info("anyone", &[]);
  let res = factory.execute(deps.as_mut(), mock_env(), info, create_pair_msg).unwrap();
  assert_eq!(
    res.attributes,
    vec![
      attr("action", "create_pair"),
      attr("pair", "wine-uusd")
    ]
  );
  assert_eq!(
    res.messages,
    vec![SubMsg {
      id: 1,
      gas_limit: None,
      msg: WasmMsg::Instantiate {
        code_id: 987u64,
        funds: vec![],
        admin: None,
        label: "".to_string(),
        msg: to_binary(&PairInstantiateMsg {
          asset_infos: asset_infos.clone(),
          token_code_id: 789u64,
          initial_price: Decimal::one(),
          tick_space: 150u16,
          fee_rate: Decimal::from_ratio(3u128, 1000u128)
        }).unwrap()
      }.into(),
      reply_on: ReplyOn::Success
    }]
  );

  assert_eq!(
    factory.temp_pair_info.load(&deps.storage).unwrap(),
    TmpPairInfo {
      pair_key: pair_key(&asset_infos.clone(), "type".to_string()),
      asset_infos: asset_infos.clone(),
      pair_type: PairType {
        type_name: "type".to_string(),
        tick_space: 150u16,
        fee_rate: Decimal::from_ratio(3u128, 1000u128)
      }
    }
  );

  // create non_uusd-non_uusd pair

  // save, wine-uusd and soju-uusd
  let asset_infos = [
    AssetInfo::Token {
      contract_addr: "wine".to_string(),
    },
    AssetInfo::NativeToken {
      denom: "uusd".to_string()
    }
  ];
  let key = pair_key(&asset_infos, "type".to_string());
  factory.pairs.save(deps.as_mut().storage, key, &PairInfo {
    asset_infos,
    contract_addr: Addr::unchecked("pair0000"),
    liquidity_token: Addr::unchecked("liquidity0000"),
    pair_type: PairType {
      type_name: "type".to_string(),
      tick_space: 150u16,
      fee_rate: Decimal::from_ratio(3u128, 1000u128)
    }
  }).unwrap();
  let asset_infos = [
    AssetInfo::Token {
      contract_addr: "soju".to_string(),
    },
    AssetInfo::NativeToken {
      denom: "uusd".to_string()
    }
  ];
  let key = pair_key(&asset_infos, "type".to_string());
  factory.pairs.save(deps.as_mut().storage, key, &PairInfo {
    asset_infos,
    contract_addr: Addr::unchecked("pair0001"),
    liquidity_token: Addr::unchecked("liquidity0001"),
    pair_type: PairType {
      type_name: "type".to_string(),
      tick_space: 150u16,
      fee_rate: Decimal::from_ratio(3u128, 1000u128)
    }
  }).unwrap();


  // make wine-soju
  let asset_infos = [
    AssetInfo::Token {
      contract_addr: "wine".to_string(),
    },
    AssetInfo::Token {
      contract_addr: "soju".to_string()
    }
  ];

  let create_pair_msg = ExecuteMsg::CreatePair {
    asset_infos: asset_infos.clone(),
    initial_price: Decimal::one(),
    pair_type: "type".to_string()
  };

  let info = mock_info("anyone", &[]);
  let res = factory.execute(deps.as_mut(), mock_env(), info, create_pair_msg).unwrap();

  assert_eq!(
    res.attributes,
    vec![
      attr("action", "create_pair"),
      attr("pair", "wine-soju")
    ]
  );
  assert_eq!(
    res.messages,
    vec![SubMsg {
      id: 1,
      gas_limit: None,
      msg: WasmMsg::Instantiate {
        code_id: 987u64,
        funds: vec![],
        admin: None,
        label: "".to_string(),
        msg: to_binary(&PairInstantiateMsg {
          asset_infos: asset_infos.clone(),
          token_code_id: 789u64,
          initial_price: Decimal::one(),
          tick_space: 150u16,
          fee_rate: Decimal::from_ratio(3u128, 1000u128)
        }).unwrap()
      }.into(),
      reply_on: ReplyOn::Success
    }]
  );

  assert_eq!(
    factory.temp_pair_info.load(&deps.storage).unwrap(),
    TmpPairInfo {
      pair_key: pair_key(&asset_infos.clone(), "type".to_string()),
      asset_infos: asset_infos.clone(),
      pair_type: PairType {
        type_name: "type".to_string(),
        tick_space: 150u16,
        fee_rate: Decimal::from_ratio(3u128, 1000u128)
      }
    }
  );
}
