use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Map;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Cw20Deposit {
    pub owner: String,
    pub amount: u128,
    pub contract: String,
    pub count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Cw721Deposit {
    pub owner: String,
    pub contract: String,
    pub token_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Deposit {
    pub owner: String,
    pub amount: Coin,
    pub count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Offer {
    pub owner: String,
    pub token_id: String,
    pub cw721_contract: String,
    pub cw20_contract: String,
    pub amount: u128,
}

//key = owner addr, denom
pub const DEPOSITS: Map<(&str, &str), Deposit> = Map::new("deposits");

//key = owner addr, contract addr
pub const CW20_DEPOSITS: Map<(&str, &str), Cw20Deposit> = Map::new("cw20deposits");

//key = owner addr, contract_addr, token_id
pub const CW721_DEPOSITS: Map<(&str, &str, &str), Cw721Deposit> = Map::new("cw721deposits");

//key = cw721 contract addr, token_id
pub const ASKS: Map<(&str, &str), Offer> = Map::new("asks");
