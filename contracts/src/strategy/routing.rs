use crate::errors::SavingsError;
use crate::storage_types::DataKey;
use crate::strategy::interface::YieldStrategyClient;
use crate::strategy::registry::{self, StrategyKey};
use crate::ttl;
use soroban_sdk::{contracttype, symbol_short, Address, Env};

/// Tracks a deposit routed to a yield strategy.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrategyPosition {
    /// The strategy contract address
    pub strategy: Address,
    /// Principal amount deposited into the strategy
    pub principal_deposited: i128,
    /// Shares received from the strategy
    pub strategy_shares: i128,
}

/// Storage key for strategy positions keyed by (plan_type_tag, plan_id).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StrategyPositionKey {
    /// Position for a LockSave plan
    Lock(u64),
    /// Position for a GroupSave plan
    Group(u64),
}

/// Routes eligible deposit funds to a registered yield strategy.
///
/// Follows the Checks-Effects-Interactions (CEI) pattern:
/// 1. **Checks** – validates strategy exists & is enabled, amount > 0
/// 2. **Effects** – persists `StrategyPosition` state
/// 3. **Interactions** – calls the external strategy contract
///
/// If the external strategy call fails, the transaction reverts atomically
/// (Soroban guarantees this), so state is always consistent.
///
/// # Arguments
/// * `env` - The contract environment
/// * `strategy_address` - Address of the target strategy contract
/// * `position_key` - Storage key for this position (Lock or Group)
/// * `amount` - Amount to deposit into the strategy
///
/// # Returns
/// The number of strategy shares received.
///
/// # Errors
/// * `StrategyNotFound` - Strategy not registered
/// * `StrategyDisabled` - Strategy is disabled
/// * `InvalidAmount` - amount <= 0
pub fn route_to_strategy(
    env: &Env,
    strategy_address: Address,
    position_key: StrategyPositionKey,
    amount: i128,
) -> Result<i128, SavingsError> {
    // --- CHECKS ---
    if amount <= 0 {
        return Err(SavingsError::InvalidAmount);
    }

    let info = registry::get_strategy(env, strategy_address.clone())?;
    if !info.enabled {
        return Err(SavingsError::StrategyDisabled);
    }

    // --- EFFECTS (state update BEFORE external call) ---
    // Optimistically record the position; Soroban atomically reverts on failure.
    let position = StrategyPosition {
        strategy: strategy_address.clone(),
        principal_deposited: amount,
        strategy_shares: 0, // placeholder, updated after call
    };
    env.storage().persistent().set(&position_key, &position);

    // --- INTERACTIONS (external call) ---
    let client = YieldStrategyClient::new(env, &strategy_address);
    let shares = client.strategy_deposit(&env.current_contract_address(), &amount);

    // Update shares after successful call
    let final_position = StrategyPosition {
        strategy: strategy_address.clone(),
        principal_deposited: amount,
        strategy_shares: shares,
    };
    env.storage()
        .persistent()
        .set(&position_key, &final_position);

    // Update global strategy principal
    let principal_key = DataKey::StrategyTotalPrincipal(strategy_address.clone());
    let current_principal: i128 = env.storage().persistent().get(&principal_key).unwrap_or(0);
    env.storage().persistent().set(
        &principal_key,
        &current_principal.checked_add(amount).unwrap(),
    );
    env.storage()
        .persistent()
        .extend_ttl(&principal_key, ttl::LOW_THRESHOLD, ttl::EXTEND_TO);

    // Extend TTL
    env.storage()
        .persistent()
        .extend_ttl(&position_key, ttl::LOW_THRESHOLD, ttl::EXTEND_TO);

    // Emit event
    env.events().publish(
        (symbol_short!("strat"), symbol_short!("deposit")),
        (strategy_address, amount, shares),
    );

    Ok(shares)
}

/// Retrieves the strategy position for a plan, if any.
pub fn get_position(env: &Env, position_key: StrategyPositionKey) -> Option<StrategyPosition> {
    env.storage().persistent().get(&position_key)
}

/// Withdraws funds from a strategy position.
///
/// # Arguments
/// * `env` - The contract environment
/// * `position_key` - The position to withdraw from
/// * `to` - The recipient address
///
/// # Returns
/// The amount of tokens received from the strategy.
pub fn withdraw_from_strategy(
    env: &Env,
    position_key: StrategyPositionKey,
    to: Address,
) -> Result<i128, SavingsError> {
    let mut position: StrategyPosition = env
        .storage()
        .persistent()
        .get(&position_key)
        .ok_or(SavingsError::StrategyNotFound)?;

    if position.principal_deposited == 0 {
        return Ok(0);
    }

    // Check strategy still exists (may be disabled, but withdrawal still allowed)
    let info_key = StrategyKey::Info(position.strategy.clone());
    if !env.storage().persistent().has(&info_key) {
        return Err(SavingsError::StrategyNotFound);
    }

    // External call: check actual balance
    let client = YieldStrategyClient::new(env, &position.strategy);
    let strategy_balance = client.strategy_balance(&env.current_contract_address());
    let withdraw_amount = position.principal_deposited.min(strategy_balance);
    if withdraw_amount <= 0 {
        return Err(SavingsError::InsufficientBalance);
    }

    // Update state BEFORE external call
    position.principal_deposited = position
        .principal_deposited
        .checked_sub(withdraw_amount)
        .ok_or(SavingsError::Underflow)?;
    position.strategy_shares = 0;
    env.storage().persistent().set(&position_key, &position);

    // Update global strategy principal
    let principal_key = DataKey::StrategyTotalPrincipal(position.strategy.clone());
    let current_principal: i128 = env.storage().persistent().get(&principal_key).unwrap_or(0);
    if current_principal >= withdraw_amount {
        env.storage()
            .persistent()
            .set(&principal_key, &(current_principal - withdraw_amount));
    } else {
        env.storage().persistent().set(&principal_key, &0_i128);
    }

    // Call strategy withdraw
    let returned = client.strategy_withdraw(&to, &withdraw_amount);

    env.events().publish(
        (symbol_short!("strat"), symbol_short!("withdraw")),
        (position.strategy, withdraw_amount, returned),
    );

    Ok(returned)
}

/// Harvests yield from a given strategy, calculates profit,
/// allocates protocol fee to treasury, and credits the rest to users.
pub fn harvest_strategy(env: &Env, strategy_address: Address) -> Result<i128, SavingsError> {
    // Check if strategy exists
    let info_key = StrategyKey::Info(strategy_address.clone());
    if !env.storage().persistent().has(&info_key) {
        return Err(SavingsError::StrategyNotFound);
    }

    let client = YieldStrategyClient::new(env, &strategy_address);
    let nestera_addr = env.current_contract_address();

    // 1. Determine current balance
    let strategy_balance = client.strategy_balance(&nestera_addr);

    // 2. Retrieve recorded principal
    let principal_key = DataKey::StrategyTotalPrincipal(strategy_address.clone());
    let principal: i128 = env.storage().persistent().get(&principal_key).unwrap_or(0);

    // 3. Calculate profit (no double counting)
    if strategy_balance <= principal {
        return Ok(0);
    }
    let profit = strategy_balance - principal;

    // 4. Call strategy harvest
    let harvested = client.strategy_harvest(&nestera_addr);

    // Safety check - we can only distribute what we actually harvested
    let actual_yield = profit.min(harvested);
    if actual_yield <= 0 {
        return Ok(0);
    }

    // 5. Calculate treasury allocation
    let config = crate::config::get_config(env)?;
    let protocol_fee_bps = config.protocol_fee_bps;

    let treasury_fee = if protocol_fee_bps > 0 {
        (actual_yield
            .checked_mul(protocol_fee_bps as i128)
            .ok_or(SavingsError::Overflow)?)
            / 10_000
    } else {
        0
    };

    let user_yield = actual_yield
        .checked_sub(treasury_fee)
        .ok_or(SavingsError::Underflow)?;

    // 6. Update accounting records
    if treasury_fee > 0 {
        let treasury_balance_key = DataKey::TotalBalance(config.treasury.clone());
        let current_treasury: i128 = env
            .storage()
            .persistent()
            .get(&treasury_balance_key)
            .unwrap_or(0);
        env.storage().persistent().set(
            &treasury_balance_key,
            &(current_treasury.checked_add(treasury_fee).unwrap()),
        );
    }

    if user_yield > 0 {
        let yield_key = DataKey::StrategyYield(strategy_address.clone());
        let current_yield: i128 = env.storage().persistent().get(&yield_key).unwrap_or(0);
        env.storage().persistent().set(
            &yield_key,
            &(current_yield.checked_add(user_yield).unwrap()),
        );
        env.storage()
            .persistent()
            .extend_ttl(&yield_key, ttl::LOW_THRESHOLD, ttl::EXTEND_TO);
    }

    env.events().publish(
        (symbol_short!("strat"), symbol_short!("harvest")),
        (strategy_address, actual_yield, treasury_fee, user_yield),
    );

    Ok(actual_yield)
}
