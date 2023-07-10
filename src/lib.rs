pub mod contract;
mod error;
pub mod state;

pub use crate::error::ContractError;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint64};

#[cw_serde]
pub struct Config {
    pub recipient: Addr,
    pub start_time: Uint64,
    pub end_time: Uint64,
    pub whitelisted_addresses: Vec<Addr>,
}

#[cw_serde]
pub struct State {
    pub last_updated_block: Uint64,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub recipient: String,
    pub start_time: Option<Uint64>,
    pub end_time: Uint64,
}

#[cw_serde]
pub enum ExecuteMsg {
    WithdrawVestedFunds,
    AddToWhitelist(AddToWhitelistMsg),
    RemoveFromWhitelist(RemoveFromWhitelistMsg),
}

#[cw_serde]
pub struct AddToWhitelistMsg {
    pub addresses: Vec<Addr>,
}

#[cw_serde]
pub struct RemoveFromWhitelistMsg {
    pub addresses: Vec<Addr>,
}

#[cw_serde]
pub enum QueryMsg {
    QueryConfig,
}
