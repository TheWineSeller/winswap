# Pair

The pair contract. Can swap, provide and withdraw with this.

## States

### config (Item)

| Key             | Description                                  |
|-----------------|----------------------------------------------|
| asset_infos     | Array of Assets that is included in pair     |
| tick_space      | smallest unit of movement in ticks           |
| fee_rate        | Swap commission rate                         |
| liquidity_token | Liquidity token address                  |
| token_code_id   | LP token's code id                           |


### tick_data (Map)
key: `tick_index` (i32)

`tick_index` is a unit tick index for this pair. If `tick_space` is 100 and `tick_index` is 10, then it will store data of price range between 1.0001^(1000) ~ 1.0001^(1100)

| Key               | Description                                   |
|-------------------|-----------------------------------------------|
| last_fee_growth_0 | Token0's accumulate commission per liqudidity |
| last_fee_growth_1 | Token1's accumulate commission per liqudidity |
| total_liquidity   | Total liquidity of this tick_index            |

### current_tick_index (Itme<i32>)
Store current_tick_index

### current_tick_index (Itme<Uint256>)
Store square root of the current price. The format is not human readable. It use Q number(Q128.128) format for calculation accuracy.

### cumulative_volume (Itme<[Uint128, Uint128]>)
Store cumulative volume to easily snapshot the volume.


## InstantiateMsg

Rust
```Rust
pub struct InstantiateMsg {
  pub asset_infos: [AssetInfo; 2],
  pub token_code_id: u64,
  pub initial_price: Decimal,
  pub tick_space: u16,
  pub fee_rate: Decimal,
}
```

Json
```json
{
  "asset_infos": [
    {"token": { "contract_addr": "terra1..." }},
    {"native_token": { "denom": "uusd" }}
  ],
  "token_code_id": 312321,
  "initial_price": "12.123",
  "tick_space": 75,
  "fee_rate": "0.0003"
}
```

## ExecuteMsg

### `Receive` (Cw20 Receive Hook)

Use Cw20's send msg for swap cw20 token to another 

Rust
```Rust
Swap {
  to: Option<String>,
  belief_price: Option<Decimal>,
  max_slippage: Option<Decimal>,
}
```

Json
```json
{
  "swap": {
    "to": "terra1...",
    "belief_price": "12.123",
    "max_slippage": "0.01"
  }
}
```

### `Swap`

Swap native Asset to another. On concentrated liquidity there is a condition that you can't swap like there are no liquidity in passing ticks.

Rust
```Rust
CreatePair {
  offer_asset: Asset,
  to: Option<String>,
  belief_price: Option<Decimal>,
  max_slippage: Option<Decimal>,
}
```

Json
```json
{
  "swap": {
    "offer_asset": {
      "info": {"native_token": { "denom": "uusd" }},
      "amount": "123123123"
    },
    "to": "terra1...",
    "belief_price": "12.123",
    "max_slippage": "0.01"
  }
}
```

### `ProvideLiquidity`

Provide liquidity. If you want new position fill the tick_indexes. If you want add liuqidity to exist position fill the token_id. When you provide to exist position the commission reward will be claimed automatically.

When you give more asset than pair want, It will return the rest.

If you provide cw20, you have to increase_allowance first

The price range is 1.0001^(tick_space * lower_tick_index) to 1.0001^(tick_space * (upper_tick_index + 1))

And I hard coded the range limit to 500 (upper_tick_index - lower_tick_index <= 500) because of the gas limit.

Rust
```Rust
ProvideLiquidity {
  assets: [Asset; 2],
  // when provide to exist position put token_id
  token_id: Option<String>,
  // when make new position put tick_indexes
  tick_indexes: Option<TickIndexes>
},
```

Json (this is just sample to show schema. you must use one btw token_id and tick_indexes)
```json
{
  "provide_liqudiity": {
    "assets": [
      {
        "info": {"native_token": { "denom": "uusd" }},
        "amount": "123123123"
      },
      {
        "info": {"token": { "contract_addr": "terra1..." }},
        "amount": "123123123"
      }
    ],
    "token_id": "123",
    "tick_indexes": {
      "upper_tick_index": 300,
      "lower_tick_index": 200
    }
  }
}
```

### `WithdrawLiquidity`

Withdraw liuqidity. If you want to partially withdraw, put amount. If you no put amount, all of the asset will be withdrawn and will burn the liquidity token. When you withdraw the commission reward will be claimed automatically.

Rust
```Rust
WithdrawLiquidity  {
  token_id: String,
  amount: Option<Uint128>
},
```

Json
```json
{
  "withdraw_liquidity": {
    "token_id": "123",
    "amount": "123123123"
  }
}
```


### `ClaimReward`

Claim reward. Only liquidity token can execute this. Rewards are calculated from lp token. User must claim reward via lp token.

Rust
```Rust
ClaimReward {
  token_id: String,
  rewards: [Asset; 2],
}
```

Json
```json
{
  "claim_reward": {
    "token_id": "123",
    "rewards": [
      {
        "info": {"native_token": { "denom": "uusd" }},
        "amount": "123123123"
      },
      {
        "info": {"token": { "contract_addr": "terra1..." }},
        "amount": "123123123"
      }
    ]
  }
}
```