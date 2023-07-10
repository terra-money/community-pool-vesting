pub mod contract;
mod error;
pub mod state;

pub use crate::error::ContractError;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint64};

#[cw_serde]
pub struct Config {
    pub recipient: Addr,
    pub end_time: Uint64,
}

#[cw_serde]
pub struct State {
    pub last_updated_block: Uint64,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub recipient: String,
    pub end_time: Uint64,
}

#[cw_serde]
pub enum ExecuteMsg {
    WithdrawVestedFunds(WithdrawVestedFundsMsg),
}

#[cw_serde]
pub struct WithdrawVestedFundsMsg {}

#[cw_serde]
pub enum QueryMsg {}
