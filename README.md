# Community Pool Vesting Contract

## Introduction

This contract is designed to manage the delegation, vesting, and withdrawal of funds based on a configurable vesting schedule. It includes functionality for `owner` and `whitelist` management. The `whitelist` is a set of users who are allowed to withdraw unlocked funds directly to the contract's specified `recipient`. This contract allows the `owner` to delegate, undelegate, redelegate, and withdraw staking rewards at *any* time, regardless of whether the funds are vested or unvested. This functionality gives the contract funds the same permissions as [Luna vesting from a genesis allocation](https://docs.terra.money/learn/protocol#vesting). 

The [instantiating parameters](#instantiatemsg) for the [contract](https://terrasco.pe/mainnet/address/terra19yxffalxzu88n5lnj40trehpryemqsz7pnnwxp8v73hxz0rl2u9q5qqwh4) defined in [governance proposal 4790](https://station.money/proposal/phoenix-1/4790) are as follows:

- `owner`: "terra159q4e7zl84hzkwy95kl29accklrxpth4zcuz8m87p4nvykpszrtq5qfgfe" (Main TFL Multisig)
- `recipient`: "terra1yv5fyftazjsy3uslzwrsaqcahn8mht87kf7jzlh50yfnu7mqxymsja06dz" (Liquidity Multisig [initially unlocked funds will be sent here])
- `unlocked_amount`: "25000000000000" (25M LUNA)
- `cliff_amount`: "25000000000000" (25M LUNA)
- `vesting_amount`: "100000000000000" (100M LUNA)
- `start_time`: "1735707600" (Jan 1, 2025, 00\:00\:00 UTC in seconds)
- `end_time`: "1861937999" (Dec 31, 2028, 23\:59\:59 UTC in seconds)

The instantiating parameters outline the following distribution:

- 25M LUNA to be immediately withdrawn to the Liquidity multisig upon the passing of the proposal. 
- 25M LUNA to be locked until fully withdrawable on Jan 1, 2025, at 00\:00\:00 UTC
- 100M LUNA to be linearly vesting from Jan 1, 2025, 00\:00\:00 UTC until Dec 31, 2028, 23\:59\:59 UTC at a rate of 68,446 (+/- 1) LUNA per day. 

### Fund Withdrawal Calculation

The fund withdrawal calculation in this contract is designed to manage the withdrawal of funds based on a vesting schedule. It includes three types of fund withdrawals: unlocked, cliff-vested, and vested. The calculation is as follows:

**Unlocked Funds:**
- `unlocked_funds` can be withdrawn at any time.
- The contract tracks the total withdrawn unlocked amount and the amount already withdrawn to ensure that the unlocked funds can only be withdrawn a single time.

**Cliff-Vested Funds:**
- `cliff_vested` funds can be wholly withdrawn after the vesting start time.
- The contract tracks the total withdrawn `cliff_vested` funds and the amount already withdrawn to ensure that the unlocked funds can only be withdrawn a single time.

**Vested Funds:**
- `vested_funds` are unlocked during the vesting period and can be withdrawn after the vesting start time.
- The contract ensures that unlocked and cliff-vested funds have been withdrawn before allowing vested fund withdrawals.
- Vested funds are calculated based on a linear vesting formula that considers the vesting start and end times. The formula is as follows:
  
  `Vested = Total Vesting Amount * (Current Time - Start Time) / (End Time - Start Time)`

❗❗❗ An important detail to note is that delegation, undelegation, redelegation, and staking reward withdrawals are **enabled**, meaning that the vesting contract gives the `owner` the same permissions as Luna vesting from genesis. ❗❗❗

## Messages

### InstantiateMsg

Purpose: This message is used to initialize the smart contract when it is first deployed. It defines the initial configuration of the contract.

**Fields:**

`owner`: The address of the contract owner, who has privileges to every single function, but does ***not*** have access to modify the contract code.
`recipient`: The address where funds can be withdrawn to.
`unlocked_amount`: The total amount of funds that are immediately unlocked and can be withdrawn at any time.
`cliff_amount`: The total amount of cliff-vested funds that become available for withdrawal after a specified cliff time.
`vesting_amount`: The total amount of funds subject to linear vesting.
`start_time`: An optional parameter representing the start time for vesting. If not provided, it defaults to the current block time.
`end_time`: The end time for vesting, after which all funds are fully vested and available for withdrawal.

### ExecuteMsg

**Purpose:** This enum defines various executable messages that can be sent to the contract to perform different actions.

**Messages:**

- [`WithdrawUnlockedFunds`](#withdraw_unlocked_funds): Initiates the withdrawal of unlocked funds.
- [`WithdrawVestedFunds`](#withdraw_vested_funds): Initiates the withdrawal of vested funds.
- [`WithdrawCliffVestedFunds`](#withdraw_cliff_vested_funds): Initiates the withdrawal of cliff-vested funds.
- [`WithdrawDelegatorReward`](#withdrawdelegatorrewardmsg): Initiates the withdrawal of rewards earned by delegating tokens to a validator.
- [`DelegateFunds`](#delegate_funds): Delegates a specified amount of tokens to a validator.
- [`UndelegateFunds`](#undelegate_funds): Undelegates a specified amount of tokens from a validator.
- [`RedelegateFunds`](#redelegate_funds): Redelegates a specified amount of tokens from one validator to another.
- [`AddToWhitelist`](#add_to_whitelist): Adds one or more addresses to the whitelist of users who can withdraw vested funds to a recipient.
- [`RemoveFromWhitelist`](#remove_from_whitelist): Removes one or more addresses from the whitelist.
- [`UpdateOwner`](#update_owner): Updates the contract owner's address.
- [`UpdateRecipient`](#update_recipient): Updates the recipient's address for fund withdrawals.

### QueryMsg

**Purpose:** This enum defines messages that can be used to query the contract's configuration or state.

**Messages:**

- `QueryConfig`: Queries the contract's configuration.
- `QueryState`: Queries the contract's state.

### WithdrawVestedFundsMsg

**Purpose:** This message is used as part of the `ExecuteMsg` to specify the details of withdrawing vested funds.

**Fields:**

- `denom`: The denomination of the token to be withdrawn (e.g., "uluna").

### WithdrawDelegatorRewardMsg

**Purpose:** This message is used as part of the `ExecuteMsg` to specify the details of withdrawing rewards earned by delegating tokens.

**Fields:**

- `validator`: The address of the validator from whom rewards are to be withdrawn.

### DelegateFundsMsg

**Purpose:** This message is used as part of the `ExecuteMsg` to specify the details of delegating funds to a validator.

**Fields:**

- `validator`: The address of the validator to whom tokens are delegated.
- `amount`: The amount of tokens to delegate, specified as a Coin object.

### UndelegateFundsMsg

**Purpose:** This message is used as part of the `ExecuteMsg` to specify the details of undelegating funds from a validator.

**Fields:**

- `validator`: The address of the validator from whom tokens are to be undelegated.
- `amount`: The amount of tokens to undelegate, specified as a Coin object.

### RedelegateFundsMsg

**Purpose:** This message is used as part of the `ExecuteMsg` to specify the details of redelegating funds from one validator to another.

**Fields:**

- `src_validator`: The address of the source validator from whom tokens are to be redelegated.
- `dst_validator`: The address of the destination validator to whom tokens are to be redelegated.
- `amount`: The amount of tokens to redelegate, specified as a Coin object.

### AddToWhitelistMsg

**Purpose:** This message is used as part of the `ExecuteMsg` to specify the addresses that should be added to the whitelist of users allowed to withdraw vested funds to the recipient.

**Fields:**

- `addresses`: A list of addresses to be added to the whitelist.

### RemoveFromWhitelistMsg

**Purpose:** This message is used as part of the `ExecuteMsg` to specify the addresses that should be removed from the whitelist of users allowed to withdraw vested funds.

**Fields:**

- `addresses`: A list of addresses to be removed from the whitelist.

### UpdateOwnerMsg

**Purpose:** This message is used as part of the `ExecuteMsg` to specify the new owner's address to update the contract's configuration.

**Fields:**

- `owner`: The new address of the contract owner.

### UpdateRecipientMsg

**Purpose:** This message is used as part of the `ExecuteMsg` to specify the new recipient's address for fund withdrawals.

**Fields:**

- `recipient`: The new address where funds can be withdrawn to.


## Functions

### `instantiate`

  **Purpose:** Instantiates the contract with the specified parameters.
  
  **Functionality:**
  - It validates the input parameters and ensures that the start time is not in the past and that the end time is after the start time.
  - Initializes and saves the contract's configuration (Config) and state (State) in the contract's storage.
  
  **Returns:** A response indicating the successful instantiation of the contract.

### `execute`

  **Purpose:** Execute functions.
  
  **Functionality:**
  - Matches the received `ExecuteMsg` with different actions and calls corresponding functions to handle them. This function delegates the execution of specific actions to other functions.
  
  **Returns:** A response based on the executed action.

### `query`

  **Purpose:** To query the contract's state or configuration.
  
  **Functionality:**
  
  - Accepts a QueryMsg to specify whether to query the contract's configuration or state.
  
  **Returns:** Binary data containing either the contract's configuration or state, based on the query.

### `update_recipient`

  **Purpose:** To update the recipient address in the contract's configuration.
  
  **Functionality:**
  
  - Checks if the sender (caller) is the contract owner. If not, it returns an error.
  - Updates the recipient address in the contract's configuration and saves it.
  
  **Returns:** A response indicating the successful update of the recipient address.

### `update_owner`

  **Purpose:** To update the contract owner's address.
  
  **Functionality:**
  
  - Checks if the sender (caller) is the current contract owner. If not, it returns an error.
  - Updates the contract owner's address and modifies the whitelist accordingly.
  
  **Returns:** A response indicating the successful update of the contract owner.

### `add_to_whitelist`

  **Purpose:** To add one or more addresses to the contract's whitelist.
  
  **Functionality:**
  - Checks if the sender (caller) is the contract owner. If not, it returns an error.
  - Adds the specified addresses to the whitelist if they are not already included.
  
  **Returns:** A response indicating the successful addition of addresses to the whitelist.

### `remove_from_whitelist`

  **Purpose:** To remove one or more addresses from the contract's whitelist.
  
  **Functionality:**
  
  - Checks if the sender (caller) is the contract owner. If not, it returns an error.
  - Removes specified addresses from the whitelist while ensuring that the recipient and owner remain whitelisted.
  
  **Returns:** A response indicating the successful removal of addresses from the whitelist.

### `delegate_funds`

  **Purpose:** To delegate (stake) funds to a validator.
  
  **Functionality:**
  
  - Checks if the sender (caller) is the contract owner. If not, it returns an error.
  - Initiates the delegation (staking) of funds to the specified validator.
  - Handles the withdrawal of delegation rewards for the validator if applicable.
  
  **Returns:** A response indicating the successful execution of the delegation action.

### `undelegate_funds`

  **Purpose:** To undelegate (unstake) funds from a validator.
  
  **Functionality:**
  
  - Checks if the sender (caller) is the contract owner. If not, it returns an error.
  - Initiates the undelegation of funds from the specified validator.
  - Handles the withdrawal of delegation rewards for the validator if applicable.
  
  **Returns:** A response indicating the successful execution of the undelegate action.

### `redelegate_funds`

  **Purpose:** To redelegate funds from one validator to another.
  
  **Functionality:**
  
  - Checks if the sender (caller) is the contract owner. If not, it returns an error.
  - Initiates redelegation of funds from the source validator to the destination validator.
  - Handles the withdrawal of delegation rewards for both validators if applicable.
  
  **Returns:** A response indicating the successful execution of the redelegate action.

### `claim_delegator_reward`

  **Purpose:** To claim delegation rewards for a validator.
  
  **Functionality:**
  
  - Checks if the sender (caller) is the contract owner. If not, it returns an error.
  - Initiates the withdrawal of delegation rewards for the specified validator.
  
  **Returns:** A response indicating the successful execution of the delegation rewards withdrawal.

### `_withdraw_delegation_rewards`

  **Purpose:** Internal function to withdraw delegation rewards for a validator.
  
  **Functionality:**
  
  - Queries the accumulated rewards for a specified validator.
  - If rewards are available, constructs a CosmosMsg to send the rewards to the contract's recipient address.
  
  **Returns:** A CosmosMsg to send rewards if available; otherwise, None.

### `withdraw_unlocked_funds`

  **Purpose:** To withdraw unlocked funds based on the vesting schedule.
  
  **Functionality:**
  
  - Checks if the sender (caller) is whitelisted. If not, it returns an error.
  - Calculates the amount of unlocked funds that can be withdrawn based on the vesting schedule.
  - Updates the state to reflect the withdrawn amount.
  - Sends the calculated amount of funds to the recipient address.
  
  **Returns:** A response indicating the successful execution of the unlocked fund withdrawal.

### `withdraw_cliff_vested_funds`

  **Purpose:** To withdraw cliff-vested funds based on the vesting schedule.

  **Functionality:**
  
  - Checks if the sender (caller) is whitelisted, the current time is after the vesting start time, and the cliff amount has not been fully withdrawn. If any of the previous are false, it returns an error.
  - Calculates the amount of cliff-vested funds that can be withdrawn based on the vesting schedule.
  - Updates the state to reflect the withdrawn amount.
  - Sends the calculated amount of funds to the recipient address.
  
  **Returns:** A response indicating the successful execution of the cliff-vested fund withdrawal.

### `withdraw_vested_funds`

  **Purpose:** To withdraw vested funds based on the vesting schedule.
  
  **Functionality:**
  
  - Checks if the sender (caller) is whitelisted and that the current time is after the vesting start time. If not, it returns an error.
  - Ensures that unlocked and cliff-vested funds have been withdrawn before allowing vested fund withdrawals.
  - Calculates the amount of vested funds that can be withdrawn based on the vesting schedule.
  - Updates the state to reflect the withdrawn amount.
  - Sends the calculated amount of funds to the recipient address.
  
  **Returns:** A response indicating the successful execution of the vested fund withdrawal.

