use crate::state::{CONFIG, STATE};
use crate::{
    AddToWhitelistMsg, Config, DelegateFundsMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
    RedelegateFundsMsg, RemoveFromWhitelistMsg, UndelegateFundsMsg, UpdateOwnerMsg,
    UpdateRecipientMsg, WithdrawDelegatorRewardMsg, WithdrawVestedFundsMsg,
};
use crate::{ContractError, State};
use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, DistributionMsg, Env,
    MessageInfo, Response, StakingMsg, StdResult, Uint128, Uint64,
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
            owner: deps.api.addr_validate(&msg.owner)?,
            recipient: deps.api.addr_validate(&msg.recipient)?,
            cliff_amount: msg.cliff_amount,
            vesting_amount: msg.vesting_amount,
            start_time: msg
                .start_time
                .clone()
                .unwrap_or(Uint64::new(env.block.time.seconds())),
            end_time: msg.end_time,
            //this whitelist is to designate users who can call the withdraw vested funds message. they cannot perform any other action
            whitelisted_addresses: vec![
                deps.api.addr_validate(&msg.owner)?,
                deps.api.addr_validate(&msg.recipient)?,
            ],
        },
    )?;

    STATE.save(
        deps.storage,
        &State {
            last_withdrawn_time: msg
                .start_time
                .unwrap_or(Uint64::new(env.block.time.seconds())),
            cliff_amount_withdrawn: Uint128::zero(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("recipient", msg.recipient)
        .add_attribute(
            "start_time",
            msg.start_time
                .unwrap_or(Uint64::new(env.block.time.seconds())),
        )
        .add_attribute("end_time", msg.end_time))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::WithdrawVestedFunds(data) => withdraw_vested_funds(deps, env, info, data),
        ExecuteMsg::WithdrawCliffVestedFunds(data) => {
            withdraw_cliff_vested_funds(deps, env, info, data)
        }
        ExecuteMsg::WithdrawDelegatorReward(data) => claim_delegator_reward(deps, env, info, data),
        ExecuteMsg::DelegateFunds(data) => delegate_funds(deps, env, info, data),
        ExecuteMsg::UndelegateFunds(data) => undelegate_funds(deps, env, info, data),
        ExecuteMsg::RedelegateFunds(data) => redelegate_funds(deps, env, info, data),
        ExecuteMsg::AddToWhitelist(data) => add_to_whitelist(deps, info, data),
        ExecuteMsg::RemoveFromWhitelist(data) => remove_from_whitelist(deps, info, data),
        ExecuteMsg::UpdateOwner(data) => update_owner(deps, info, data),
        ExecuteMsg::UpdateRecipient(data) => update_recipient(deps, info, data),
    }
}

fn update_recipient(
    deps: DepsMut,
    info: MessageInfo,
    data: UpdateRecipientMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    CONFIG.save(
        deps.storage,
        &Config {
            owner: config.owner,
            recipient: deps.api.addr_validate(&data.recipient)?,
            cliff_amount: config.cliff_amount,
            vesting_amount: config.vesting_amount,
            start_time: config.start_time,
            end_time: config.end_time,
            whitelisted_addresses: config.whitelisted_addresses,
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "update_recipient")
        .add_attribute("owner", format!("{:?}", data.recipient)))
}

fn update_owner(
    deps: DepsMut,
    info: MessageInfo,
    data: UpdateOwnerMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&data.owner)?,
            recipient: config.recipient,
            cliff_amount: config.cliff_amount,
            vesting_amount: config.vesting_amount,
            start_time: config.start_time,
            end_time: config.end_time,
            whitelisted_addresses: config.whitelisted_addresses,
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "update_owner")
        .add_attribute("owner", format!("{:?}", data.owner)))
}

fn remove_from_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    data: RemoveFromWhitelistMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    //always keep recipient and owner address on the whitelist
    let mut new_addresses = vec![config.owner.clone(), config.recipient.clone()];
    for addr in config.whitelisted_addresses {
        if !data.addresses.contains(&addr) && addr != config.recipient && addr != config.owner {
            new_addresses.push(addr);
        }
    }
    CONFIG.save(
        deps.storage,
        &Config {
            owner: config.owner,
            recipient: config.recipient,
            cliff_amount: config.cliff_amount,
            vesting_amount: config.vesting_amount,
            start_time: config.start_time,
            end_time: config.end_time,
            whitelisted_addresses: new_addresses.clone(),
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "remove_from_whitelist")
        .add_attribute("whitelisted_addresses", format!("{:?}", new_addresses)))
}

fn add_to_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    data: AddToWhitelistMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
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
            owner: config.owner,
            recipient: config.recipient,
            cliff_amount: config.cliff_amount,
            vesting_amount: config.vesting_amount,
            start_time: config.start_time,
            end_time: config.end_time,
            whitelisted_addresses: new_addresses.clone(),
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "add_to_whitelist")
        .add_attribute("whitelisted_addresses", format!("{:?}", new_addresses)))
}

fn redelegate_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: RedelegateFundsMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let msg = CosmosMsg::Staking(StakingMsg::Redelegate {
        src_validator: data.src_validator.clone(),
        dst_validator: data.dst_validator.clone(),
        amount: data.amount.clone(),
    });
    let send_reward_msg_src =
        _withdraw_delegation_rewards(&deps.as_ref(), &env, &info, &data.src_validator);
    let send_reward_msg_dst =
        _withdraw_delegation_rewards(&deps.as_ref(), &env, &info, &data.dst_validator);

    let mut res = Response::new()
        .add_message(msg)
        .add_attribute("action", "redelegate_funds")
        .add_attribute("src_validator", data.src_validator)
        .add_attribute("dst_validator", data.dst_validator)
        .add_attribute("denom", data.amount.denom)
        .add_attribute("amount", data.amount.amount);

    if let Some(send_reward_msg) = send_reward_msg_src {
        res = res.add_message(send_reward_msg);
    }
    if let Some(send_reward_msg) = send_reward_msg_dst {
        res = res.add_message(send_reward_msg);
    }

    Ok(res)
}

fn undelegate_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: UndelegateFundsMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let msg = CosmosMsg::Staking(StakingMsg::Undelegate {
        validator: data.validator.clone(),
        amount: data.amount.clone(),
    });
    let send_reward_msg =
        _withdraw_delegation_rewards(&deps.as_ref(), &env, &info, &data.validator);

    let mut res = Response::new()
        .add_message(msg)
        .add_attribute("action", "undelegate_funds")
        .add_attribute("validator", data.validator)
        .add_attribute("denom", data.amount.denom)
        .add_attribute("amount", data.amount.amount);

    if let Some(send_reward_msg) = send_reward_msg {
        res = res.add_message(send_reward_msg);
    }
    Ok(res)
}

fn claim_delegator_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: WithdrawDelegatorRewardMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let msg = CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
        validator: data.validator.clone(),
    });
    let send_reward_msg =
        _withdraw_delegation_rewards(&deps.as_ref(), &env, &info, &data.validator);

    let mut res = Response::new()
        .add_message(msg)
        .add_attribute("action", "withdraw_delegator_rewards")
        .add_attribute("validator", data.validator);

    if let Some(send_reward_msg) = send_reward_msg {
        res = res.add_message(send_reward_msg);
    }
    Ok(res)
}

fn delegate_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: DelegateFundsMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let msg = CosmosMsg::Staking(StakingMsg::Delegate {
        validator: data.validator.clone(),
        amount: data.amount.clone(),
    });
    let send_reward_msg =
        _withdraw_delegation_rewards(&deps.as_ref(), &env, &info, &data.validator);

    let mut res = Response::new()
        .add_message(msg)
        .add_attribute("action", "delegate_funds")
        .add_attribute("validator", data.validator)
        .add_attribute("denom", data.amount.denom)
        .add_attribute("amount", data.amount.amount);

    if let Some(send_reward_msg) = send_reward_msg {
        res = res.add_message(send_reward_msg);
    }

    Ok(res)
}

fn _withdraw_delegation_rewards(
    deps: &Deps,
    env: &Env,
    info: &MessageInfo,
    validator: &String,
) -> Option<CosmosMsg> {
    let delegation_result = deps
        .querier
        .query_delegation(env.contract.address.to_string(), validator);
    if let Ok(delegation) = delegation_result {
        if let Some(delegation) = delegation {
            let rewards: Vec<Coin> = delegation.accumulated_rewards.iter().filter(|r| !r.amount.is_zero()).cloned().collect();
            if rewards.is_empty() {
                return None;
            }
            return Some(CosmosMsg::Bank(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: rewards,
            }));
        }
    }
    return None;
}

fn withdraw_cliff_vested_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: WithdrawVestedFundsMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    if !config.whitelisted_addresses.contains(&info.sender)
        || env.block.time.seconds() < config.start_time.u64()
        || state.cliff_amount_withdrawn >= config.cliff_amount
    {
        return Err(ContractError::Unauthorized {});
    }

    let current_balance = deps
        .querier
        .query_balance(env.contract.address.clone(), data.denom.clone())?
        .amount;

    let amount_to_withdraw = if data.denom == "uluna" {
        let withdrawable = current_balance.min(config.cliff_amount - state.cliff_amount_withdrawn);
        state.cliff_amount_withdrawn += withdrawable;
        STATE.save(deps.storage, &state)?;
        withdrawable
    } else {
        current_balance
    };

    if amount_to_withdraw.is_zero() {
        return Err(ContractError::NothingToWithdraw {});
    }

    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: config.recipient.to_string(),
        amount: vec![Coin::new(amount_to_withdraw.u128(), data.denom.clone())],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "withdraw_vested_funds")
        .add_attribute("denom", data.denom)
        .add_attribute("amount_to_withdraw", amount_to_withdraw)
        .add_attribute(
            "cliff_amount_withdrawn",
            state.cliff_amount_withdrawn.clone(),
        ))
}

fn withdraw_vested_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: WithdrawVestedFundsMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    if !config.whitelisted_addresses.contains(&info.sender)
        || env.block.time.seconds() < config.start_time.u64()
    {
        return Err(ContractError::Unauthorized {});
    }

    // Need to withdraw cliff amount before linear vesting amount
    if state.cliff_amount_withdrawn < config.cliff_amount {
        return Err(ContractError::WithdrawCliffFirst {});
    }

    let current_balance = deps
        .querier
        .query_balance(env.contract.address.clone(), data.denom.clone())?
        .amount;

    // Use end time here to prevent overflow
    let current_time = config.end_time.u64().min(env.block.time.seconds());

    let amount_withdrawable = config.vesting_amount
        - config.vesting_amount
            * Uint128::from(state.last_withdrawn_time.u64() - config.start_time.u64())
            / Uint128::from(config.end_time - config.start_time)
        - config.vesting_amount * Uint128::from(config.end_time.u64() - current_time)
            / Uint128::from(config.end_time - config.start_time);

    let balance_smaller_than_withdrawable = if current_time < config.end_time.u64() {
        current_balance < amount_withdrawable
    } else {
        true
    };

    let mut amount_to_withdraw = if data.denom == "uluna" {
        if balance_smaller_than_withdrawable {
            current_balance
        } else {
            amount_withdrawable
        }
    } else {
        current_balance
    };

    let last_withdrawn_time = if balance_smaller_than_withdrawable {
        //if balance is smaller than withdrawable, we set the withdrawn time in seconds to something smaller than the current blocktime
        state.last_withdrawn_time
            + Uint64::try_from(
                amount_to_withdraw
                    / (config.vesting_amount / Uint128::from(config.end_time - config.start_time)),
            )?
    } else {
        Uint64::new(current_time)
    };

    STATE.save(
        deps.storage,
        &State {
            last_withdrawn_time: if data.denom == "uluna" {
                //only update the withdrawal block if the asset withdrawn is luna
                last_withdrawn_time
            } else {
                state.last_withdrawn_time
            },
            cliff_amount_withdrawn: state.cliff_amount_withdrawn,
        },
    )?;

    if amount_to_withdraw.is_zero() {
        return Err(ContractError::NothingToWithdraw {});
    }

    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: config.recipient.to_string(),
        amount: vec![Coin::new(amount_to_withdraw.u128(), data.denom.clone())],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "withdraw_vested_funds")
        .add_attribute("denom", data.denom)
        .add_attribute("amount_to_withdraw", amount_to_withdraw)
        .add_attribute("last_updated_block", env.block.time.seconds().to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::QueryState => to_binary(&STATE.load(deps.storage)?),
    }
}
