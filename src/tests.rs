use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{Addr, BankMsg, BlockInfo, Coin, coin, ContractInfo, CosmosMsg, Env, MessageInfo, OwnedDeps, ReplyOn, SubMsg, Uint128, Uint64};
use crate::contract::{execute, instantiate};
use crate::{ExecuteMsg, InstantiateMsg, WithdrawVestedFundsMsg};

fn instantiate_contract() -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, Env, MessageInfo, MessageInfo) {
    let mut deps = mock_dependencies();
    let owner = mock_info("vlad", &[coin(1_000_000, "uluna")]);
    let recipient = mock_info("javier", &[]);
    let env = Env {
        block: BlockInfo {
            height: 0,
            time: Default::default(),
            chain_id: "phoenix-1".to_string(),
        },
        transaction: None,
        contract: ContractInfo {
            address: Addr::unchecked("community_pool_vesting_contract")
        },
    };

    let instantiate_msg = InstantiateMsg {
        owner: owner.sender.to_string(),
        recipient: recipient.clone().sender.to_string(),
        initial_amount: Uint128::new(1_000_000),
        start_time: Some(Uint64::zero()),
        end_time: Uint64::new(100),
    };

    instantiate(deps.as_mut(), env.clone(), owner.clone(), instantiate_msg).unwrap();

    (deps, env, owner, recipient)
}

#[test]
fn test_withdraw_vested_funds_owner() {
    let (mut deps, mut env, mut owner, recipient) = instantiate_contract();
    owner.funds = vec![];
    env.block.time = env.block.time.plus_seconds(10);

    let res = execute(deps.as_mut(), env, owner, ExecuteMsg::WithdrawVestedFunds(WithdrawVestedFundsMsg {
        denom: "uluna".to_string(),
    })).unwrap();

    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.sender.to_string(),
                amount: vec![Coin::new(10_000, "uluna")],
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
fn test_withdraw_vested_funds_before_vesting_started() {
    todo!()
}

#[test]
fn test_withdraw_vested_funds_zero_balance() {
    todo!()
}

#[test]
fn test_withdraw_vested_funds_balance_smaller_than_withdrawable() {
    todo!()
}

#[test]
fn test_withdraw_vested_funds_balance_larger_than_withdrawable() {
    todo!()
}

#[test]
fn test_withdraw_vested_funds_balance_equal_to_withdrawable() {
    todo!()
}

#[test]
fn test_withdraw_vested_funds_balance_vesting_ended() {
    todo!()
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