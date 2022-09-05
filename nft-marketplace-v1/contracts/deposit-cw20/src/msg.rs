use cw20::Cw20ReceiveMsg;

use cw721::Cw721ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{Cw20Deposit, Cw721Deposit, Deposit};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Deposit {},
    Withdraw {
        amount: u128,
        denom: String,
    },
    Receive(Cw20ReceiveMsg),
    ReceiveNft(Cw721ReceiveMsg),
    WithdrawCw20 {
        owner: String,
        amount: u128,
    },
    WithdrawNft {
        cw721_contract: String,
        token_id: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetCw20Deposit { address: String },
    GetDeposits { address: String },
    GetCw721Deposit { address: String, contract: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Deposit {
        owner: String,
        amount: u128,
    },
    Purchase {
        token_id: String,
        cw721_contract: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw721HookMsg {
    Deposit {
        owner: String,
        token_id: String,
        cw20_contract: String,
        amount: u128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Cw20DepositResponse {
    pub deposits: Vec<Cw20Deposit>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DepositResponse {
    pub deposits: Vec<Deposit>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Cw721DepositResponse {
    pub deposits: Vec<Cw721Deposit>,
}
