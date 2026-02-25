use crate::errors::SavingsError;
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
    let position: StrategyPosition = env
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

    // Update state BEFORE external call
    let cleared = StrategyPosition {
        strategy: position.strategy.clone(),
        principal_deposited: 0,
        strategy_shares: 0,
    };
    env.storage().persistent().set(&position_key, &cleared);

    // External call
    let client = YieldStrategyClient::new(env, &position.strategy);
    let returned = client.strategy_withdraw(&to, &position.principal_deposited);

    env.events().publish(
        (symbol_short!("strat"), symbol_short!("withdraw")),
        (position.strategy, returned),
    );

    Ok(returned)
}
