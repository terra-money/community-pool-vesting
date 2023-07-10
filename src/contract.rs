use crate::state::{CONFIG, STATE};
use crate::{Config, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::{ContractError, State};
use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, Uint64,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    CONFIG.save(
        deps.storage,
        &Config {
            recipient: deps.api.addr_validate(&msg.recipient)?,
            start_time: msg.start_time.clone().unwrap_or(Uint64::new(env.block.time.seconds())),
            end_time: msg.end_time,
            //this whitelist is to designate users who can call the withdraw vested funds message. they cannot perform any other action
            whitelisted_addresses: vec![deps.api.addr_validate(&msg.recipient)?],
        },
    )?;

    STATE.save(
        deps.storage,
        &State {
            last_updated_block: msg.start_time.unwrap_or(Uint64::new(env.block.time.seconds())),
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("recipient", msg.recipient)
        .add_attribute("start_time", msg.start_time.unwrap_or(Uint64::new(env.block.time.seconds())))
        .add_attribute("end_time", msg.end_time))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    if !config.whitelisted_addresses.contains(&info.sender) || env.block.time.seconds() < config.start_time.u64() {
        return Err(ContractError::Unauthorized {});
    }
    match msg {
        ExecuteMsg::WithdrawVestedFunds => {
            if !config.whitelisted_addresses.contains(&info.sender) || env.block.time.seconds() < config.start_time.u64() {
                return Err(ContractError::Unauthorized {});
            }
            let amount_to_withdraw = deps
                .querier
                .query_balance(env.contract.address, "uluna")?
                .amount
                / Uint128::from(config.end_time - config.start_time)
                * Uint128::from(env.block.time.seconds() - state.last_updated_block.u64());

            STATE.save(
                deps.storage,
                &State {
                    last_updated_block: Uint64::new(env.block.time.seconds()),
                },
            )?;

            let msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: config.recipient.to_string(),
                amount: vec![Coin::new(amount_to_withdraw.u128(), "uluna")],
            });

            Ok(Response::new()
                .add_message(msg)
                .add_attribute("action", "withdraw_vested_funds")
                .add_attribute("amount_to_withdraw", amount_to_withdraw)
                .add_attribute("last_updated_block", env.block.time.seconds().to_string()))
        }
        ExecuteMsg::AddToWhitelist(data) => {
            if config.recipient != info.sender || env.block.time.seconds() < config.start_time.u64() {
                return Err(ContractError::Unauthorized {});
            }
            let mut new_addresses = config.whitelisted_addresses.clone();
            for addr in data.addresses {
                if !config.whitelisted_addresses.contains(&addr) {
                    new_addresses.push(addr)
                }
            }
            CONFIG.save(
                deps.storage,
                &Config {
                    recipient: config.recipient,
                    start_time: config.start_time,
                    end_time: config.end_time,
                    whitelisted_addresses: new_addresses,
                }
            )?;
            Ok(Response::new()
                .add_attribute("action", "add_to_whitelist")
                .add_attribute("whitelisted_addresses", format!("{:?}", new_addresses))
            )
        }
        ExecuteMsg::RemoveFromWhitelist(data) => {
            if config.recipient != info.sender || env.block.time.seconds() < config.start_time.u64() {
                return Err(ContractError::Unauthorized {});
            }
            //always keep recipient address on the whitelist
            let mut new_addresses = vec![config.recipient];
            for addr in config.whitelisted_addresses {
                if !data.addresses.contains(&addr) && addr != config.recipient {
                    new_addresses.push(addr);
                }
            }
            CONFIG.save(
                deps.storage,
                &Config {
                    recipient: config.recipient,
                    start_time: config.start_time,
                    end_time: config.end_time,
                    whitelisted_addresses: new_addresses,
                }
            )?;
            Ok(Response::new()
                .add_attribute("action", "remove_from_whitelist")
                .add_attribute("whitelisted_addresses", format!("{:?}", new_addresses))
            )
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig => {
            to_binary(&CONFIG.load(deps.storage)?)
        }
    }
}
