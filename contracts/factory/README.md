# Factory

The factory contract. Create pair, store pair list and some data

## States

### config (Item)

| Key            | Description                                  |
|----------------|----------------------------------------------|
| owner          | Only owner can update config, add pair type  |
| pair_code_id   | Pair's code id                               |
| token_code_id  | LP token's code id                           |


### pair_type (Map)

| Key            | Description                                  |
|----------------|----------------------------------------------|
| type_name(key) | Type's name                                  |
| tick_space     | Pair type's tick move amount.                |
| fee_rate       | Swap commission rate                         |

### pairs (IndexedMap)
key: sort(asset_infos) + pair_type

| Key                | Description                                  |
|--------------------|----------------------------------------------|
| asset_infos(index) | Array of Assets that is included in pair     |
| contract_addr      | Pair contract address                        |
| liquidity_token    | Liquidity token address                      |
| pair_type          | Pair type's name                             |

### temp_pair_info (Item)
Store temporary pair info to use replied data

| Key                | Description                                  |
|--------------------|----------------------------------------------|
| pair_key           | Key of pair: sort(asset_infos) + pair_type   |
| asset_infos        | Array of Assets that is included in pair     |
| pair_type          | Pair type's name                             |


## InstantiateMsg

Rust
```Rust
pub struct InstantiateMsg {
  pub owner: String,
  pub pair_code_id: u64,
  pub token_code_id: u64,
}
```

Json
```json
{
  "owner": "terra1...",
  "pair_code_id": 123123,
  "token_code_id": 312321
}
```

## ExecuteMsg

### `UpdateConfig`

Update config. Only owner can execute this

Rust
```Rust
UpdateConfig {
  owner: Option<String>,
  token_code_id: Option<u64>,
  pair_code_id: Option<u64>,
}
```

Json
```json
{
  "update_config": {
    "owner": "terra1...",
    "token_code_id": 321312,
    "pair_code_id": 1242132
  }
}
```

### `CreatePair`

Create pair with given info. Can't make pair with same asset_infos and pair_type. Everyone can execute this.

`initial_price` is `token0` price as `token1`. So I highly recommend put UST to `token1` like below json example

Rust
```Rust
CreatePair {
  asset_infos: [AssetInfo; 2],
  pair_type: String,
  initial_price: Decimal,
}
```

Json
```json
{
  "create_pair": {
    "asset_infos": [
      {"token": { "contract_addr": "terra1..." }},
      {"native_token": { "denom": "uusd" }}
    ],
    "pair_type": "normal",
    "initial_price": "12.123",
  }
}
```

### `AddPairType`

Add pair type. Only owner can execute this.

`tick_space` is the smallest unit of movement in ticks. So user only can provide liquidity between the price like below

price_range: (1.0001)^(`tick_space` * `m`) ~ (1.0001)^(`tick_space` * `n`)

`m` and `n` are integer

Rust
```Rust
AddPairType {
  type_name: String,
  tick_space: u16,
  fee_rate: Decimal,
},
```

Json
```json
{
  "add_pair_type": {
    "type_name": "normal",
    "tick_space": 75,
    "fee_rate": "0.003",
  }
}
```