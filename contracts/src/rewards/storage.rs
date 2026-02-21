use super::storage_types::{RewardsDataKey, UserRewards};
use crate::errors::SavingsError;
use crate::rewards::config::get_rewards_config;
use crate::rewards::storage_types::RewardsConfig;
use soroban_sdk::{symbol_short, Address, Env, Symbol};

/// Duration threshold for long-lock bonus eligibility (in seconds).
pub const LONG_LOCK_BONUS_THRESHOLD_SECS: u64 = 180 * 24 * 60 * 60;

/// Fetches user rewards or returns a default empty state
pub fn get_user_rewards(env: &Env, user: Address) -> UserRewards {
    let key = RewardsDataKey::UserLedger(user);

    // Automatically extend TTL on read to prevent data expiry
    if let Some(rewards) = env
        .storage()
        .persistent()
        .get::<RewardsDataKey, UserRewards>(&key)
    {
        env.storage().persistent().extend_ttl(&key, 17280, 17280); // ~1 day extension
        rewards
    } else {
        UserRewards {
            total_points: 0,
            lifetime_deposited: 0,
            current_streak: 0,
            last_action_timestamp: 0,
        }
    }
}

/// Force-saves the user rewards state
pub fn save_user_rewards(env: &Env, user: Address, rewards: &UserRewards) {
    let key = RewardsDataKey::UserLedger(user);
    env.storage().persistent().set(&key, rewards);
    env.storage().persistent().extend_ttl(&key, 17280, 17280);
}

pub fn initialize_user_rewards(env: &Env, user: Address) -> Result<(), SavingsError> {
    let key = RewardsDataKey::UserLedger(user.clone());

    if env.storage().persistent().has(&key) {
        return Err(SavingsError::UserAlreadyExists);
    }

    let initial_rewards = UserRewards {
        total_points: 0,
        lifetime_deposited: 0,
        current_streak: 0,
        last_action_timestamp: env.ledger().timestamp(),
    };

    // Now this function can find save_user_rewards because they are in the same file
    save_user_rewards(env, user, &initial_rewards);
    Ok(())
}

/// Increases user points with overflow protection
pub fn add_points(env: &Env, user: Address, points: u128) -> Result<(), SavingsError> {
    let mut rewards = get_user_rewards(env, user.clone());

    // Safety check for overflow
    rewards.total_points = rewards
        .total_points
        .checked_add(points)
        .ok_or(SavingsError::Overflow)?;

    save_user_rewards(env, user, &rewards);
    Ok(())
}

/// Resets the streak back to zero
pub fn reset_streak(env: &Env, user: Address) {
    let mut rewards = get_user_rewards(env, user.clone());
    rewards.current_streak = 0;
    save_user_rewards(env, user, &rewards);
}

pub fn award_deposit_points(env: &Env, user: Address, amount: i128) -> Result<(), SavingsError> {
    // 1. Fetch Config & Check if Enabled
    let config = get_rewards_config(env)?;
    if !config.enabled {
        return Ok(()); // Zero impact when disabled
    }

    // 2. Fetch User State
    let mut user_rewards = get_user_rewards(env, user.clone());

    // 3. Calculate Base Points
    // Using checked_mul to prevent overflow during calculation
    let base_points = (amount as u128)
        .checked_mul(config.points_per_token as u128)
        .ok_or(SavingsError::Overflow)?;

    // 4. Update State
    user_rewards.total_points = user_rewards
        .total_points
        .checked_add(base_points)
        .ok_or(SavingsError::Overflow)?;

    user_rewards.lifetime_deposited = user_rewards
        .lifetime_deposited
        .checked_add(amount)
        .ok_or(SavingsError::Overflow)?;

    // 5. Save and Emit Event
    save_user_rewards(env, user.clone(), &user_rewards);

    env.events().publish(
        (symbol_short!("rewards"), symbol_short!("awarded"), user),
        base_points,
    );

    Ok(())
}

/// Awards bonus points for long lock plans when duration exceeds the configured threshold.
pub fn award_long_lock_bonus(
    env: &Env,
    user: Address,
    amount: i128,
    duration: u64,
) -> Result<u128, SavingsError> {
    if amount <= 0 || duration <= LONG_LOCK_BONUS_THRESHOLD_SECS {
        return Ok(0);
    }

    let config = match get_rewards_config(env) {
        Ok(config) if config.enabled => config,
        _ => return Ok(0),
    };

    if config.long_lock_bonus_bps == 0 || config.points_per_token == 0 {
        return Ok(0);
    }

    let base_points = (amount as u128)
        .checked_mul(config.points_per_token as u128)
        .ok_or(SavingsError::Overflow)?;
    let bonus_points = base_points
        .checked_mul(config.long_lock_bonus_bps as u128)
        .ok_or(SavingsError::Overflow)?
        / 10_000u128;

    if bonus_points == 0 {
        return Ok(0);
    }

    add_points(env, user.clone(), bonus_points)?;
    env.events().publish(
        (Symbol::new(env, "BonusAwarded"), user, symbol_short!("lock")),
        bonus_points,
    );
    Ok(bonus_points)
}

/// Awards a fixed goal completion bonus when a goal reaches its target.
pub fn award_goal_completion_bonus(env: &Env, user: Address) -> Result<u128, SavingsError> {
    let config = match get_rewards_config(env) {
        Ok(config) if config.enabled => config,
        _ => return Ok(0),
    };

    if config.goal_completion_bonus == 0 {
        return Ok(0);
    }

    let bonus_points = config.goal_completion_bonus as u128;
    add_points(env, user.clone(), bonus_points)?;
    env.events().publish(
        (Symbol::new(env, "BonusAwarded"), user, symbol_short!("goal")),
        bonus_points,
    );
    Ok(bonus_points)
}
