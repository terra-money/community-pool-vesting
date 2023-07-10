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
            start_time: msg.start_time.unwrap_or(Uint64::new(env.block.height)),
            end_time: msg.end_time,
        },
    )?;

    STATE.save(
        deps.storage,
        &State {
            last_updated_block: Uint64::new(env.block.height),
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("recipient", msg.recipient)
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
    if info.sender != config.recipient || env.block.height < config.start_time.u64() {
        return Err(ContractError::Unauthorized {});
    }
    match msg {
        ExecuteMsg::WithdrawVestedFunds(_) => {
            let amount_to_withdraw = deps
                .querier
                .query_balance(env.contract.address, "uluna")?
                .amount
                / Uint128::from(config.end_time - config.start_time)
                * Uint128::from(env.block.height - state.last_updated_block.u64());

            STATE.save(
                deps.storage,
                &State {
                    last_updated_block: Uint64::new(env.block.height),
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
                .add_attribute("last_updated_block", env.block.height.to_string()))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    to_binary("")
}
