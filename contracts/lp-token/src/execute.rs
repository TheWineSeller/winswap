
use cosmwasm_std::{to_binary, Binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg};
use cw0::Expiration;
use crate::error::ContractError;
use wineswap::lp_token::{LpReceiveMsg, ConfigResponse, InstantiateMsg, ExecuteMsg};
use wineswap::pair::ExecuteMsg as PairExecuteMsg;
use crate::state::{LiquidityInfo, LpContract, FeeInfo, Approval};

impl<'a> LpContract<'a> {
  pub fn instantiate(
    &self,
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
  ) -> StdResult<Response> {
    let config = ConfigResponse{
      name: msg.name,
      symbol: msg.symbol,
      minter: deps.api.addr_validate(&msg.minter)?,
    };
  
    self.config.save(deps.storage, &config)?;
    Ok(Response::default())
  }

  pub fn execute(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
  ) -> Result<Response, ContractError> {
    match msg {
      ExecuteMsg::Mint {
        owner,
        liquidity,
        upper_tick_index,
        lower_tick_index
      } => self.mint(deps, env, info, owner, liquidity, upper_tick_index, lower_tick_index),
      ExecuteMsg::Burn {
        token_id
      } => self.burn(deps, env, info, token_id),
      ExecuteMsg::Approve {
        spender,
        token_id,
        expires,
      } => self.approve(deps, env, info, spender, token_id, expires),
      ExecuteMsg::Revoke { spender, token_id } => {
        self.revoke(deps, env, info, spender, token_id )
      },
      ExecuteMsg::Transfer {
        recipient,
        token_id,
      } => self.transfer(deps, env, info, recipient, token_id),
      ExecuteMsg::Send {
        contract,
        token_id,
        msg,
      } => self.send(deps, env, info, contract, token_id, msg),
      ExecuteMsg::ClaimReward {
        token_id,
      } => self.claim_reward(deps, env, info, token_id),
      ExecuteMsg::UpdateLiquidity {
        token_id,
        amount,
        add,
      } => self.update_liquidity(deps, env, info, token_id, amount, add)
    } 
  }
}


/// execute function
impl<'a> LpContract<'a> {
  pub fn mint(
    &self,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: String,
    liquidity: Uint128,
    upper_tick_index: i32,
    lower_tick_index: i32
  ) -> Result<Response, ContractError> {
    let config = self.config.load(deps.storage)?;
    let minter = config.minter;
  
    // only minter can mint
    if info.sender != minter {
      return Err(ContractError::Unauthorized {});
    }
  
    let token_count = self.token_count.load(deps.storage).unwrap_or(0);
  
    // get current fee_infos from pair contract
    let fee_infos = self.get_fee_infos(deps.querier, minter.to_string(), upper_tick_index, lower_tick_index)?;
  
    let token = LiquidityInfo {
      owner: deps.api.addr_validate(&owner)?,
      approvals: vec![],
      liquidity,
      upper_tick_index,
      lower_tick_index,
      last_updated_fee_infos: fee_infos.clone()
    };
  
    self.tokens
      .update(deps.storage, &token_count.to_string(), |old| match old {
        Some(_) => Err(ContractError::Claimed {}),
        None => Ok(token),
      })?;
  
    self.increment_tokens(deps.storage)?;
  
    Ok(Response::new()
      .add_attribute("action", "mint_liquidity")
      .add_attribute("owner", owner)
      .add_attribute("token_id", token_count.to_string())
    )
  }

  pub fn approve(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
    expires: Option<Expiration>
  ) -> Result<Response, ContractError> {
    let mut token = self.tokens.load(deps.storage, &token_id)?;

    // only owner can apporve
    if token.owner != info.sender {
      return Err(ContractError::Unauthorized {});
    }

    let spender_addr = deps.api.addr_validate(&spender)?;

    // update the approval list (remove any for the same spender before adding)
    token.approvals = token
        .approvals
        .into_iter()
        .filter(|apr| apr.spender != spender_addr)
        .collect();

    let expires = expires.unwrap_or_default();

    // if expired
    if expires.is_expired(&env.block) {
      return Err(ContractError::Expired {});
    }

    let approval = Approval {
      spender: spender_addr,
      expires,
    };
    token.approvals.push(approval);

    self.tokens.save(deps.storage, &token_id, &token)?;

    Ok(Response::new()
      .add_attribute("action", "approve")
      .add_attribute("sender", info.sender)
      .add_attribute("spender", spender)
      .add_attribute("token_id", token_id)
    )
  }

  pub fn revoke(
    &self,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
  ) -> Result<Response, ContractError> {
    let mut token = self.tokens.load(deps.storage, &token_id)?;

    if token.owner != info.sender {
      return Err(ContractError::Unauthorized {});
    }


    let spender_addr = deps.api.addr_validate(&spender)?;
    token.approvals = token
        .approvals
        .into_iter()
        .filter(|apr| apr.spender != spender_addr)
        .collect();


    self.tokens.save(deps.storage, &token_id, &token)?;

    Ok(Response::new()
      .add_attribute("action", "revoke")
      .add_attribute("sender", info.sender)
      .add_attribute("spender", spender)
      .add_attribute("token_id", token_id)
    )
  }

  pub fn transfer(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
  ) -> Result<Response, ContractError> {
    let token = self.tokens.load(deps.storage, &token_id)?; 
    let owner = token.owner;

    self._transfer(deps, &env, &info, &recipient, &token_id)?;
    Ok(Response::new()
      .add_attribute("action", "transfer")
      .add_attribute("from", owner)
      .add_attribute("to", recipient)
      .add_attribute("token_id", token_id)
    )
  }

  pub fn send(
    &self,
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    token_id: String,
    msg: Binary,
  ) -> Result<Response, ContractError> {
    let token = self.tokens.load(deps.storage, &token_id)?;
    let owner = token.owner;

    self._transfer(deps, &env, &info, &contract, &token_id)?;

    let send = LpReceiveMsg {
      sender: info.sender.to_string(),
      token_id: token_id.clone(),
      msg,
    };

    Ok(Response::new()
      .add_message(send.into_cosmos_msg(contract.clone())?)
      .add_attribute("action", "send")
      .add_attribute("from", owner)
      .add_attribute("to", contract)
      .add_attribute("token_id", token_id)
    )
  }

  pub fn burn(
    &self,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
  ) -> Result<Response, ContractError> {
    let config = self.config.load(deps.storage)?;
    let minter = config.minter;

    // only minter(pair) can burn
    if info.sender != minter {
      return Err(ContractError::Unauthorized {});
    }
    
    self.tokens.remove(deps.storage, &token_id)?;

    Ok(Response::new()
      .add_attribute("action", "burn")
      .add_attribute("token_id", token_id)
    )
  }


  pub fn claim_reward(
    &self,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
  ) -> Result<Response, ContractError> {
    let mut token = self.tokens.load(deps.storage, &token_id)?; 
    let owner = token.owner.clone();
    let config = self.config.load(deps.storage)?;

    // only owner and pair(for additional provide and partial withdraw) can execute
    if !(owner == info.sender || config.minter == info.sender) {
      return Err(ContractError::Unauthorized {});
    } 

    let minter = config.minter.clone();

    let new_infos: Vec<FeeInfo> = self.get_fee_infos(deps.querier, minter.to_string(), token.upper_tick_index, token.lower_tick_index)?;

    let reward = self.reward(deps.as_ref(), token_id.clone())?;

    // fee infos update
    token.last_updated_fee_infos = new_infos;

    self.tokens.save(deps.storage, &token_id, &token)?;

    let claim_msg = CosmosMsg::Wasm(WasmMsg::Execute {
      contract_addr: config.minter.to_string(),
      msg: to_binary(&PairExecuteMsg::ClaimReward {
        token_id: token_id.clone(),
        rewards: reward.rewards,
      })?,
      funds: vec![],
    });

    Ok(Response::new().add_message(claim_msg))
  }

  pub fn update_liquidity(
    &self,
    deps: DepsMut, 
    _env: Env,
    info: MessageInfo,
    token_id: String,
    amount: Uint128,
    add: bool
  ) -> Result<Response, ContractError> {
    let mut token = self.tokens.load(deps.storage, &token_id)?; 
    let config = self.config.load(deps.storage)?;
    // Only minter(pair) can execute this
    if config.minter != info.sender {
      return Err(ContractError::Unauthorized {})
    }

    // update liquidity
    // additional provide
    if add {
      token.liquidity = token.liquidity + amount;
    // partial withdraw
    } else {
      token.liquidity = token.liquidity.checked_sub(amount)?;
    }

    // save token
    self.tokens.save(deps.storage, &token_id, &token)?;

    Ok(Response::new())
  }
}

/// help function
impl<'a> LpContract<'a> {
  fn _transfer(
    &self,
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    recipient: &str, 
    token_id: &str
  ) -> Result<LiquidityInfo, ContractError> {
    let mut token = self.tokens.load(deps.storage, &token_id)?;

    self.check_can_send(&env, &info, &token)?;

    token.owner = deps.api.addr_validate(recipient)?;

    // reset approvals
    token.approvals = vec![];
    self.tokens.save(deps.storage, &token_id, &token)?;
    Ok(token)
  }

  fn check_can_send(
    &self,
    env: &Env,
    info: &MessageInfo,
    token: &LiquidityInfo,
  ) -> Result<(), ContractError> {
    if token.owner == info.sender {
      return Ok(());
    }
    
    // check approvals
    if token
        .approvals
        .iter()
        .any(|apr| apr.spender == info.sender && !apr.is_expired(&env.block))
      {
        return Ok(());
      }

    Err(ContractError::Unauthorized {})
  }
}
