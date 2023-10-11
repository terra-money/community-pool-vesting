use crate::contract::{execute, instantiate, query};
use crate::state::{CONFIG, STATE};
use crate::{
    AddToWhitelistMsg, Config, ContractError, DelegateFundsMsg, ExecuteMsg, InstantiateMsg,
    QueryMsg, RedelegateFundsMsg, RemoveFromWhitelistMsg, State, UndelegateFundsMsg,
    UpdateOwnerMsg, UpdateRecipientMsg, WithdrawDelegatorRewardMsg, WithdrawVestedFundsMsg,
};
use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage, StakingQuerier,
    MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::{
    coin, from_binary, Addr, BankMsg, BlockInfo, Coin, ContractInfo, CosmosMsg, DepsMut,
    DistributionMsg, Empty, Env, MessageInfo, OwnedDeps, ReplyOn, Response, StakingMsg, SubMsg,
    Timestamp, Uint128, Uint64,
};
use std::marker::PhantomData;

const CONTRACT_ADDR: &str = "community_pool_vesting_contract";
const VESTING_START_TIME: u64 = 1735707600; //jan 1, 2025, 00:00:00
const VESTING_END_TIME: u64 = 1861937999; //dec 31, 2028, 23:59:59

const CLIFF_AMOUNT: u128 = 25_000_000_000_000; //25m u_units
const VESTING_AMOUNT: u128 = 100_000_000_000_000; //100m u_units

const VESTED_PER_DAY: u128 = 68_446_270_221; //leap year and 3 years (incl. rounding errors from inner calculation) rounding error is a fraction of a luna, so it is within tolerance (actual value == 68446269678 == VESTING_AMOUNT/(365*3+366)
const VESTED_PER_SECOND: u128 = VESTED_PER_DAY / 24 / 60 / 60;

const DAY_IN_SECONDS: u64 = 86400;



fn mock_dependencies_with_contract_balance(
    amount: Uint128,
) -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    let mut mock_querier = MockQuerier::new(&[(
        CONTRACT_ADDR,
        &[Coin {
            denom: "uluna".to_string(),
            amount,
        }],
    )]);
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: mock_querier,
        custom_query_type: PhantomData,
    }
}

fn instantiate_contract() -> (
    OwnedDeps<MockStorage, MockApi, MockQuerier>,
    Env,
    MessageInfo,
    MessageInfo,
) {
    let mut deps = mock_dependencies_with_contract_balance(Uint128::new(1_100_000));
    let owner = mock_info("vlad", &[]);
    let recipient = mock_info("javier", &[]);
    let env = Env {
        block: BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(VESTING_START_TIME-1),
            chain_id: "phoenix-1".to_string(),
        },
        transaction: None,
        contract: ContractInfo {
            address: Addr::unchecked(CONTRACT_ADDR),
        },
    };

    let instantiate_msg = InstantiateMsg {
        owner: owner.sender.to_string(),
        recipient: recipient.clone().sender.to_string(),
        cliff_amount: Uint128::new(CLIFF_AMOUNT),
        vesting_amount: Uint128::new(VESTING_AMOUNT),
        start_time: Some(Uint64::new(VESTING_START_TIME)),
        end_time: Uint64::new(VESTING_END_TIME),
    };

    deps.querier
        .update_balance(CONTRACT_ADDR, vec![Coin::new(VESTING_AMOUNT + CLIFF_AMOUNT, "uluna")]); // prefill contract with community pool funds

    instantiate(deps.as_mut(), env.clone(), owner.clone(), instantiate_msg).unwrap();

    (deps, env, owner, recipient)
}

#[test]
fn test_withdraw_vested_funds_owner() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(DAY_IN_SECONDS+1);

    let mut res = execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::WithdrawCliffVestedFunds(WithdrawVestedFundsMsg {
            denom: "uluna".to_string(),
        }),
    )
    .unwrap();
    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(CLIFF_AMOUNT, "uluna")],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    res = execute(
        deps.as_mut(),
        env,
        owner,
        ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
            denom: "uluna".to_string(),
        }),
    )
    .unwrap();

    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(VESTED_PER_DAY, "uluna")], //one day of vesting
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );
}

#[test]
fn test_withdraw_vested_funds_whitelist() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(DAY_IN_SECONDS+1);

    let mut res = execute(
        deps.as_mut(),
        env.clone(),
        recipient.clone(),
        ExecuteMsg::WithdrawCliffVestedFunds(WithdrawVestedFundsMsg {
            denom: "uluna".to_string(),
        }),
    )
    .unwrap();
    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(CLIFF_AMOUNT, "uluna")],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );
    deps.querier
        .update_balance(CONTRACT_ADDR, vec![Coin::new(VESTING_AMOUNT, "uluna")]);

    res = execute(
        deps.as_mut(),
        env,
        recipient.clone(),
        ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
            denom: "uluna".to_string(),
        }),
    )
    .unwrap();

    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(VESTED_PER_DAY, "uluna")],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );
}

#[test]
fn test_withdraw_vested_funds_before_withdrawing_cliff_vested() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(10);

    let res = execute(
        deps.as_mut(),
        env,
        owner,
        ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
            denom: "uluna".to_string(),
        }),
    )
    .unwrap_err();

    assert_eq!(res, ContractError::WithdrawCliffFirst {});
}

#[test]
fn test_withdraw_cliff_vested_funds_with_not_enough_balance() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(1);

    let mut res = execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::WithdrawCliffVestedFunds(WithdrawVestedFundsMsg {
            denom: "uluna".to_string(),
        }),
    )
    .unwrap();
    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(CLIFF_AMOUNT, "uluna")],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    let state = STATE.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        state,
        State {
            cliff_amount_withdrawn: Uint128::new(CLIFF_AMOUNT),
            last_withdrawn_time: Uint64::new(VESTING_START_TIME),
        }
    );

    // Withdrawing vested funds should fail until all cliff vested funds are withdrawn
    execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
            denom: "uluna".to_string(),
        }),
    )
    .unwrap_err();
}

#[test]
fn test_withdraw_vested_funds_before_vesting_started() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.minus_seconds(5);

    execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::WithdrawCliffVestedFunds(WithdrawVestedFundsMsg {
            denom: "uluna".to_string(),
        }),
    )
    .unwrap_err();
}

#[test]
fn test_withdraw_vested_funds_zero_balance() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(1);
    deps.querier
        .update_balance(CONTRACT_ADDR, vec![Coin::new(0, "uluna")]);

    let mut res = execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::WithdrawCliffVestedFunds(WithdrawVestedFundsMsg {
            denom: "uluna".to_string(),
        }),
    )
    .unwrap_err();
    assert_eq!(res, ContractError::NothingToWithdraw {});

    STATE
        .save(
            deps.as_mut().storage,
            &State {
                cliff_amount_withdrawn: Uint128::new(CLIFF_AMOUNT),
                last_withdrawn_time: Uint64::new(VESTING_START_TIME),
            },
        )
        .unwrap();

    let res = execute(
        deps.as_mut(),
        env,
        owner,
        ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
            denom: "uluna".to_string(),
        }),
    )
    .unwrap_err();

    assert_eq!(res, ContractError::NothingToWithdraw {});
}

#[test]
fn test_withdraw_vested_funds_balance_smaller_than_withdrawable() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(DAY_IN_SECONDS * 2 + 1);

    STATE
        .save(
            deps.as_mut().storage,
            &State {
                last_withdrawn_time: Uint64::new(VESTING_START_TIME),
                cliff_amount_withdrawn: Uint128::new(CLIFF_AMOUNT),
            },
        )
        .unwrap(); //cliff withdrawn

    deps.querier
        .update_balance(CONTRACT_ADDR, vec![Coin::new(VESTED_PER_DAY, "uluna")]); //amount per second

    let res = execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
            denom: "uluna".to_string(),
        }),
    )
    .unwrap();

    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(VESTED_PER_DAY, "uluna")],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    let state = STATE.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        state,
        State {
            last_withdrawn_time: Uint64::new(VESTING_START_TIME + DAY_IN_SECONDS), //1 second worth withdraw, move the needle by 1 second
            cliff_amount_withdrawn: Uint128::new(25000000000000),
        }
    );

    deps.querier
        .update_balance(CONTRACT_ADDR, vec![Coin::new(150_000_000_000, "uluna")]); //now more is in the balance than is withdrawable
    let res = execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
            denom: "uluna".to_string(),
        }),
    )
    .unwrap();

    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(VESTED_PER_DAY + 1, "uluna")], //withdraws daily amount minus already withdrawn second worth of tokens in the day (+1 uluna needed for the rounding error)
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    let state = STATE.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        state,
        State {
            last_withdrawn_time: Uint64::new(VESTING_START_TIME + DAY_IN_SECONDS * 2),
            cliff_amount_withdrawn: Uint128::new(CLIFF_AMOUNT),
        }
    );
}

#[test]
fn test_withdraw_vested_funds_balance_vesting_ended() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = Timestamp::from_seconds(VESTING_END_TIME+1); //past the end of vesting

    STATE
        .save(
            deps.as_mut().storage,
            &State {
                last_withdrawn_time: Uint64::new(VESTING_START_TIME),
                cliff_amount_withdrawn: Uint128::new(CLIFF_AMOUNT),
            },
        )
        .unwrap();

    deps.querier.update_balance(CONTRACT_ADDR, vec![coin(VESTING_AMOUNT, "uluna")]); // assume cliff has been withdrawn

    let res = execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
            denom: "uluna".to_string(),
        }),
    )
    .unwrap();

    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(VESTING_AMOUNT, "uluna")],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );
}

#[test]
fn test_withdraw_vested_funds_balance_non_luna() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    deps.querier.update_balance(
        CONTRACT_ADDR,
        vec![Coin::new(1_000_000, "uusd")],
    );

    let res = execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::WithdrawCliffVestedFunds(WithdrawVestedFundsMsg {
            denom: "uusd".to_string(),
        }),
    )
    .unwrap();

    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(1_000_000, "uusd")],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    STATE
        .save(
            deps.as_mut().storage,
            &State {
                last_withdrawn_time: Uint64::new(VESTING_START_TIME),
                cliff_amount_withdrawn: Uint128::new(CLIFF_AMOUNT),
            },
        )
        .unwrap();

    let res = execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
            denom: "uusd".to_string(),
        }),
    )
    .unwrap();

    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(1000_000, "uusd")],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );
}

#[test]
fn test_withdraw_vested_funds_unauthorized() {
    //neither owner nor whitelist
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    deps.querier
        .update_balance(CONTRACT_ADDR, vec![Coin::new(1000_000, "uluna")]);

    let info = mock_info("random", &[]);

    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::WithdrawCliffVestedFunds(WithdrawVestedFundsMsg {
            denom: "uluna".to_string(),
        }),
    )
    .unwrap_err();

    assert_eq!(res, ContractError::Unauthorized {});

    STATE
        .save(
            deps.as_mut().storage,
            &State {
                last_withdrawn_time: Uint64::new(10),
                cliff_amount_withdrawn: Uint128::new(100_000),
            },
        )
        .unwrap();

    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
            denom: "uluna".to_string(),
        }),
    )
    .unwrap_err();

    assert_eq!(res, ContractError::Unauthorized {},);
}

#[test]
fn test_add_to_whitelist_successful() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::AddToWhitelist(AddToWhitelistMsg {
            addresses: vec![Addr::unchecked("warp")],
        }),
    )
    .unwrap();

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config.whitelisted_addresses,
        vec![
            Addr::unchecked("vlad"),
            Addr::unchecked("javier"),
            Addr::unchecked("warp")
        ]
    );
}

#[test]
fn test_add_to_whitelist_not_owner() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    let info = mock_info("random", &[]);

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::AddToWhitelist(AddToWhitelistMsg {
            addresses: vec![Addr::unchecked("warp")],
        }),
    )
    .unwrap_err();
}

#[test]
fn test_add_to_whitelist_already_included() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::AddToWhitelist(AddToWhitelistMsg {
            addresses: vec![Addr::unchecked("javier")],
        }),
    )
    .unwrap();

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config.whitelisted_addresses,
        vec![Addr::unchecked("vlad"), Addr::unchecked("javier")]
    );
}

#[test]
fn test_remove_from_whitelist_successful() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::AddToWhitelist(AddToWhitelistMsg {
            addresses: vec![Addr::unchecked("warp")],
        }),
    )
    .unwrap();

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config.whitelisted_addresses,
        vec![
            Addr::unchecked("vlad"),
            Addr::unchecked("javier"),
            Addr::unchecked("warp")
        ]
    );

    execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::RemoveFromWhitelist(RemoveFromWhitelistMsg {
            addresses: vec![Addr::unchecked("warp")],
        }),
    )
    .unwrap();

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config.whitelisted_addresses,
        vec![Addr::unchecked("vlad"), Addr::unchecked("javier")]
    );
}

#[test]
fn test_remove_from_whitelist_not_owner() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    let info = mock_info("random", &[]);

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::RemoveFromWhitelist(RemoveFromWhitelistMsg {
            addresses: vec![Addr::unchecked("warp")],
        }),
    )
    .unwrap_err();
}

#[test]
fn test_remove_recipient_from_whitelist() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::RemoveFromWhitelist(RemoveFromWhitelistMsg {
            addresses: vec![recipient.sender.clone()],
        }),
    )
    .unwrap();

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config.whitelisted_addresses,
        vec![owner.sender, recipient.sender]
    );
}

#[test]
fn test_remove_owner_from_whitelist() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::RemoveFromWhitelist(RemoveFromWhitelistMsg {
            addresses: vec![owner.sender.clone()],
        }),
    )
    .unwrap();

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config.whitelisted_addresses,
        vec![owner.sender, recipient.sender]
    );
}

#[test]
fn test_update_owner_successful() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::UpdateOwner(UpdateOwnerMsg {
            owner: "rando".to_string(),
        }),
    )
    .unwrap();

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config.owner, Addr::unchecked("rando"));
}

#[test]
fn test_update_owner_unauthorized() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    let info = mock_info("random", &[]);

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateOwner(UpdateOwnerMsg {
            owner: "rando".to_string(),
        }),
    )
    .unwrap_err();
}

#[test]
fn test_update_recipient_successful() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::UpdateRecipient(UpdateRecipientMsg {
            recipient: "rando".to_string(),
        }),
    )
    .unwrap();

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config.recipient, Addr::unchecked("rando"));
}

#[test]
fn test_update_recipient_unauthorized() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    let info = mock_info("random", &[]);

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateRecipient(UpdateRecipientMsg {
            recipient: "rando".to_string(),
        }),
    )
    .unwrap_err();
}

#[test]
fn test_delegate_funds_successful() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    let res = execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::DelegateFunds(DelegateFundsMsg {
            validator: "random".to_string(),
            amount: Coin::new(100_000, "uluna"),
        }),
    )
    .unwrap();

    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Staking(StakingMsg::Delegate {
            validator: "random".to_string(),
            amount: Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(100_000),
            },
        }))
    );
}

#[test]
fn test_delegate_funds_unauthorized() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    let info = mock_info("random", &[]);

    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::DelegateFunds(DelegateFundsMsg {
            validator: "random".to_string(),
            amount: Coin::new(100_000, "uluna"),
        }),
    )
    .unwrap_err();
    assert_eq!(res, ContractError::Unauthorized {});
}

#[test]
fn test_undelegate_funds_successful() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    let res = execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::UndelegateFunds(UndelegateFundsMsg {
            validator: "random".to_string(),
            amount: Coin::new(100_000, "uluna"),
        }),
    )
    .unwrap();

    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Staking(StakingMsg::Undelegate {
            validator: "random".to_string(),
            amount: Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(100_000),
            },
        }))
    );
}

#[test]
fn test_undelegate_funds_unauthorized() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    let info = mock_info("random", &[]);

    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UndelegateFunds(UndelegateFundsMsg {
            validator: "random".to_string(),
            amount: Coin::new(100_000, "uluna"),
        }),
    )
    .unwrap_err();
    assert_eq!(res, ContractError::Unauthorized {});
}

#[test]
fn test_redelegate_funds_successful() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    let res = execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::RedelegateFunds(RedelegateFundsMsg {
            src_validator: "random".to_string(),
            dst_validator: "another".to_string(),
            amount: Coin::new(100_000, "uluna"),
        }),
    )
    .unwrap();

    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Staking(StakingMsg::Redelegate {
            src_validator: "random".to_string(),
            dst_validator: "another".to_string(),
            amount: Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(100_000),
            },
        }))
    );
}

#[test]
fn test_redelegate_funds_unauthorized() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    let info = mock_info("random", &[]);

    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::RedelegateFunds(RedelegateFundsMsg {
            src_validator: "random".to_string(),
            dst_validator: "another".to_string(),
            amount: Coin::new(100_000, "uluna"),
        }),
    )
    .unwrap_err();
    assert_eq!(res, ContractError::Unauthorized {});
}

#[test]
fn test_withdraw_delegator_reward_successful() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    let res = execute(
        deps.as_mut(),
        env.clone(),
        owner.clone(),
        ExecuteMsg::WithdrawDelegatorReward(WithdrawDelegatorRewardMsg {
            validator: "random".to_string(),
        }),
    )
    .unwrap();

    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Distribution(
            DistributionMsg::WithdrawDelegatorReward {
                validator: "random".to_string(),
            }
        ))
    );
}

#[test]
fn test_withdraw_delegator_reward_unauthorized() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    let info = mock_info("random", &[]);

    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::WithdrawDelegatorReward(WithdrawDelegatorRewardMsg {
            validator: "random".to_string(),
        }),
    )
    .unwrap_err();
    assert_eq!(res, ContractError::Unauthorized {});
}

#[test]
fn test_query_config() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    let res = query(deps.as_ref(), env.clone(), QueryMsg::QueryConfig {}).unwrap();
    let value: Config = from_binary(&res).unwrap();
    assert_eq!(
        value,
        Config {
            owner: owner.sender.clone(),
            recipient: recipient.sender.clone(),
            cliff_amount: Uint128::new(CLIFF_AMOUNT),
            vesting_amount: Uint128::new(VESTING_AMOUNT),
            start_time: Uint64::new(VESTING_START_TIME),
            end_time: Uint64::new(VESTING_END_TIME),
            whitelisted_addresses: vec![owner.sender, recipient.sender],
        }
    );
}

#[test]
fn test_query_state() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    let res = query(deps.as_ref(), env.clone(), QueryMsg::QueryState {}).unwrap();
    let value: State = from_binary(&res).unwrap();

    assert_eq!(
        value,
        State {
            last_withdrawn_time: Uint64::new(1735707600),
            cliff_amount_withdrawn: Uint128::new(0),
        }
    );
}
