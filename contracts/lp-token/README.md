# LP Token

The Lp Token contract. Nearly same with cw721. User can transfer token to other and claim commission reward to it.

## States

### config (Item)

| Key             | Description                                  |
|-----------------|----------------------------------------------|
| name            | Token name (hard coded at pair)              |
| symbol          | Token symbol (hard coded at pair)            |
| minter          | Address that can mint, pair in this case     |


### token_count (Itme<u64>)
Store token index and use it for token_id of the next minted token and increase 1

### tokens (IndexedMap)
key: token_id (u64)

| Key                    | Description                                                          |
|------------------------|----------------------------------------------------------------------|
| owner(index)           | Owner of the token                                                   |
| liquidity              | Token's liquidity amount                                             |
| upper_tick_index       | Token's upper tick index. (See pair for more detail)                 |
| lower_tick_index       | Token's lower tick index. (See pair for more detail)                 |
| last_updated_fee_infos | Copy of fee infos from pair at the moment of thelast raward claim    |
| approvals              | Approvls                                                             |


## InstantiateMsg

Rust
```Rust
pub struct InstantiateMsg {
  pub name: String,
  pub symbol: String,
  pub minter: String,
}
```

Json
```json
{
  "name": "lp token",
  "symbol": "LP",
  "minter": "terra1...",
}
```

## ExecuteMsg

### `Transfer`

Transfer token to other.

Rust
```Rust
Transfer {
  recipient: String,
  token_id: String 
}
```

Json
```json
{
  "transfer": {
    "recipient": "terra1...",
    "token_id": "123",
  }
}
```

### `Send`

Send token to contract and execute msg

Rust
```Rust
Send { 
  contract: String,
  token_id: String,
  msg: Binary 
},
```

Json
```json
{
  "send": {
    "conatract": "terra1...",
    "token_id": "123",
    "msg": "eyJzb21ldGhpbmciOnt9fQ==",
  }
}
```

### `Approve`

Give approval to spender. Approvals will be canceled when it transfer or send to other

Rust
```Rust
Approve {
  spender: String,
  token_id: String,
  expires: Option<Expiration>,
}
```

Json
```json
{
  "apporve": {
    "spender": "terra1...",
    "token_id": "123",
    "expires": {
      "never": {}
    }
  }
}
```

### `Revokde`

Cancel approval

Rust
```Rust
Revoke { 
  spender: String,
  token_id: String
},
```

Json
```json
{
  "revoke": {
    "spender": "terra1...",
    "token_id": "123"
  }
}
```


### `Burn`

Burn token. Only minter(pair) can execute this.

Rust
```Rust
Burn { 
  token_id: String
}
```

Json
```json
{
  "burn": {
    "token_id": "123",
  }
}
```

### `Mint`

Mint token. Only minter(pair) can execute this.

Rust
```Rust
  Mint {
    owner: String,
    liquidity: Uint128,
    upper_tick_index: i32,
    lower_tick_index: i32
  },
```

Json
```json
{
  "mint": {
    "owner": "terra1...",
    "liquidity": "123123",
    "uppder_tick_index": 300,
    "lower_tick_index": 200
  }
}
```

### `ClaimReward`

Claim the commission reward. Only owner can exectue this.

Rust
```Rust
  ClaimReward {
    token_id: "123"
  }
```

Json
```json
{
  "claim_reward": {
    "token_id": "123"
  }
}
```

### `UpdateLiquidity`

Modify liquidity amount. Only minter(pair) can execute this for additional provide or partial withdraw.
If add is true, add liquidity, if false sub liquidity

Rust
```Rust
  UpdateLiquidity {
    token_id: String,
    amount: Uint128,
    add: bool
  }
```

Json
```json
{
  "update_liquidity": {
    "token_id": "123",
    "amount": "123123",
    "add": false 
  }
}
```