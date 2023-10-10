use std::marker::PhantomData;
use cosmwasm_std::testing::{MOCK_CONTRACT_ADDR, mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{Addr, BankMsg, BlockInfo, Coin, coin, ContractInfo, CosmosMsg, DepsMut, Empty, Env, MessageInfo, OwnedDeps, ReplyOn, Response, SubMsg, Timestamp, Uint128, Uint64};
use crate::contract::{execute, instantiate};
use crate::{ContractError, ExecuteMsg, InstantiateMsg, State, WithdrawVestedFundsMsg};
use crate::state::STATE;

const CONTRACT_ADDR: &str = "community_pool_vesting_contract";

fn mock_dependencies_with_contract_balance(amount: Uint128) -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    let mut mock_querier = MockQuerier::new(&[(
        CONTRACT_ADDR,
        &[Coin {
            denom: "uluna".to_string(),
            amount,
        }]
    )]);
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: mock_querier,
        custom_query_type: PhantomData,
    }
}

fn instantiate_contract() -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, Env, MessageInfo, MessageInfo) {
    let mut deps = mock_dependencies_with_contract_balance(Uint128::new(1_100_000));
    let owner = mock_info("vlad", &[coin(1_100_000, "uluna")]);
    let recipient = mock_info("javier", &[]);
    let env = Env {
        block: BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(10),
            chain_id: "phoenix-1".to_string(),
        },
        transaction: None,
        contract: ContractInfo {
            address: Addr::unchecked(CONTRACT_ADDR)
        },
    };

    let instantiate_msg = InstantiateMsg {
        owner: owner.sender.to_string(),
        recipient: recipient.clone().sender.to_string(),
        cliff_amount: Uint128::new(100_000),
        vesting_amount: Uint128::new(1_000_000),
        start_time: Some(Uint64::new(10)),
        end_time: Uint64::new(110),
    };

    instantiate(deps.as_mut(), env.clone(), owner.clone(), instantiate_msg).unwrap();

    (deps, env, owner, recipient)
}

#[test]
fn test_withdraw_vested_funds_owner() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(10);

    let mut res = execute(deps.as_mut(), env.clone(), owner.clone(), ExecuteMsg::WithdrawCliffVestedFunds(WithdrawVestedFundsMsg {
        denom: "uluna".to_string(),
    })).unwrap();
    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(100_000, "uluna")],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );
    deps.querier.update_balance(CONTRACT_ADDR, vec![Coin::new(1_000_000, "uluna")]);

    res = execute(deps.as_mut(), env, owner, ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
        denom: "uluna".to_string(),
    })).unwrap();

    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(100_000, "uluna")],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );
}

#[test]
fn test_withdraw_vested_funds_whitelist() {
    todo!()
}

#[test]
fn test_withdraw_vested_funds_before_withdrawing_cliff_vested() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(10);

    let res = execute(deps.as_mut(), env, owner, ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
        denom: "uluna".to_string(),
    })).unwrap_err();

    assert_eq!(
        res,
        ContractError::Unauthorized {}
    );
}

#[test]
fn test_withdraw_cliff_vested_funds_with_not_enough_balance() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(10);

    deps.querier.update_balance(CONTRACT_ADDR, vec![Coin::new(50_000, "uluna")]);
    let mut res = execute(deps.as_mut(), env.clone(), owner.clone(), ExecuteMsg::WithdrawCliffVestedFunds(WithdrawVestedFundsMsg {
        denom: "uluna".to_string(),
    })).unwrap();
    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(50_000, "uluna")],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    let state = STATE.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        state,
        State {
            cliff_amount_withdrawn: Uint128::new(50_000),
            last_withdrawn_time: Uint64::new(10),
        }
    );

    // Withdrawing vested funds should fail until all cliff vested funds are withdrawn
    execute(deps.as_mut(), env.clone(), owner.clone(), ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
        denom: "uluna".to_string(),
    })).unwrap_err();
}

#[test]
fn test_withdraw_vested_funds_before_vesting_started() {
    todo!()
}

#[test]
fn test_withdraw_vested_funds_zero_balance() {
    todo!()
}

#[test]
fn test_withdraw_vested_funds_balance_smaller_than_withdrawable() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(10);

    STATE.save(deps.as_mut().storage, &State {
        last_withdrawn_time: Uint64::new(10),
        cliff_amount_withdrawn: Uint128::new(100_000),
    }).unwrap();
    deps.querier.update_balance(CONTRACT_ADDR, vec![Coin::new(50_000, "uluna")]);

    let res = execute(deps.as_mut(), env.clone(), owner.clone(), ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
        denom: "uluna".to_string(),
    })).unwrap();

    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(50_000, "uluna")],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    let state = STATE.load(deps.as_ref().storage).unwrap();
    assert_eq!(state,
        State {
            last_withdrawn_time: Uint64::new(15),
            cliff_amount_withdrawn: Uint128::new(100_000),
        }
    );

    deps.querier.update_balance(CONTRACT_ADDR, vec![Coin::new(150_000, "uluna")]);
    let res = execute(deps.as_mut(), env.clone(), owner.clone(), ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
        denom: "uluna".to_string(),
    })).unwrap();

    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(50_000, "uluna")],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    let state = STATE.load(deps.as_ref().storage).unwrap();
    assert_eq!(state,
               State {
                   last_withdrawn_time: Uint64::new(20),
                   cliff_amount_withdrawn: Uint128::new(100_000),
               }
    );
}

#[test]
fn test_withdraw_vested_funds_balance_vesting_ended() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(200);

    STATE.save(deps.as_mut().storage, &State {
        last_withdrawn_time: Uint64::new(10),
        cliff_amount_withdrawn: Uint128::new(100_000),
    }).unwrap();
    deps.querier.update_balance(CONTRACT_ADDR, vec![Coin::new(1000_000, "uluna")]);

    let res = execute(deps.as_mut(), env.clone(), owner.clone(), ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
        denom: "uluna".to_string(),
    })).unwrap();

    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(1000_000, "uluna")],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );
}

#[test]
fn test_withdraw_vested_funds_balance_non_luna() {
    todo!()
}

#[test]
fn test_withdraw_vested_funds_unauthorized() { //neither owner nor whitelist
    todo!()
}

#[test]
fn test_add_to_whitelist_successful() {
    todo!()
}

#[test]
fn test_add_to_whitelist_not_owner() {
    todo!()
}

#[test]
fn test_add_to_whitelist_already_included() {
    todo!()
}

#[test]
fn test_remove_from_whitelist_successful() {
    todo!()
}

#[test]
fn test_remove_from_whitelist_not_owner() {
    todo!()
}

#[test]
fn test_remove_recipient_from_whitelist() {
    todo!()
}

#[test]
fn test_remove_owner_from_whitelist() {
    todo!()
}

#[test]
fn test_update_owner_successful() {
    todo!()
}

#[test]
fn test_update_owner_unauthorized() {
    todo!()
}

#[test]
fn test_update_recipient_successful() {
    todo!()
}

#[test]
fn test_update_recipient_unauthorized() {
    todo!()
}

#[test]
fn test_delegate_funds_successful() {
    todo!()
}

#[test]
fn test_delegate_funds_unauthorized() {
    todo!()
}

#[test]
fn test_undelegate_funds_successful() {
    todo!()
}

#[test]
fn test_undelegate_funds_unauthorized() {
    todo!()
}

#[test]
fn test_redelegate_funds_successful() {
    todo!()
}

#[test]
fn test_redelegate_funds_unauthorized() {
    todo!()
}

#[test]
fn test_withdraw_delegator_reward_successful() {
    todo!()
}

#[test]
fn test_withdraw_delegator_reward_unauthorized() {
    todo!()
}

#[test]
fn test_query_config() {
    todo!()
}

#[test]
fn test_query_state() {
    todo!()
}