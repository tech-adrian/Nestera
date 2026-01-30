use crate::errors::SavingsError;
use crate::storage_types::{DataKey, LockSave, User};
use crate::ensure_not_paused;
use crate::users;
use soroban_sdk::{symbol_short, Address, Env, Vec};

/// Creates a new Lock Save plan for a user
pub fn create_lock_save(
    env: &Env,
    user: Address,
    amount: i128,
    duration: u64,
) -> Result<u64, SavingsError> {
    ensure_not_paused(env)?;
    user.require_auth();

    // Validate inputs
    if amount <= 0 {
        return Err(SavingsError::InvalidAmount);
    }
    if duration == 0 {
        // Aligned with the test expectation of a generic invalid duration error
        return Err(SavingsError::InvalidTimestamp);
    }

    // Ensure user exists using your users module
    if !users::user_exists(env, &user) {
        return Err(SavingsError::UserNotFound);
    }

    // ID Logic
    let lock_id = get_next_lock_id(env);
    increment_next_lock_id(env);

    let start_time = env.ledger().timestamp();
    let maturity_time = start_time
        .checked_add(duration)
        .ok_or(SavingsError::Overflow)?;

    let lock_save = LockSave {
        id: lock_id,
        owner: user.clone(),
        amount,
        interest_rate: 500, // Matching your test expectation of 500 (5%)
        start_time,
        maturity_time,
        is_withdrawn: false,
    };

    // Store the LockSave
    env.storage()
        .persistent()
        .set(&DataKey::LockSave(lock_id), &lock_save);

    // Update user's lock list
    add_lock_to_user(env, &user, lock_id);

    // Update user's profile stats
    let user_key = DataKey::User(user.clone());
    let mut user_data: User = env.storage().persistent().get(&user_key).unwrap();
    user_data.total_balance += amount;
    user_data.savings_count += 1;
    env.storage().persistent().set(&user_key, &user_data);

    Ok(lock_id)
}

pub fn withdraw_lock_save(env: &Env, user: Address, lock_id: u64) -> Result<i128, SavingsError> {
    ensure_not_paused(env)?;
    user.require_auth();

    let mut lock_save = get_lock_save(env, lock_id).ok_or(SavingsError::PlanNotFound)?;

    if lock_save.owner != user {
        return Err(SavingsError::Unauthorized);
    }

    if lock_save.is_withdrawn {
        return Err(SavingsError::PlanCompleted);
    }

    if !check_matured_lock(env, lock_id) {
        return Err(SavingsError::TooEarly);
    }

    let final_amount = calculate_lock_save_yield(&lock_save, env.ledger().timestamp());

    lock_save.is_withdrawn = true;
    env.storage()
        .persistent()
        .set(&DataKey::LockSave(lock_id), &lock_save);

    // Update user's total balance (subtracting the locked portion)
    let user_key = DataKey::User(user.clone());
    if let Some(mut user_data) = env.storage().persistent().get::<DataKey, User>(&user_key) {
        user_data.total_balance -= lock_save.amount;
        env.storage().persistent().set(&user_key, &user_data);
    }

    env.events()
        .publish((symbol_short!("withdraw"), user, lock_id), final_amount);

    Ok(final_amount)
}

pub fn check_matured_lock(env: &Env, lock_id: u64) -> bool {
    if let Some(lock_save) = get_lock_save(env, lock_id) {
        env.ledger().timestamp() >= lock_save.maturity_time
    } else {
        false
    }
}

pub fn get_lock_save(env: &Env, lock_id: u64) -> Option<LockSave> {
    env.storage().persistent().get(&DataKey::LockSave(lock_id))
}

pub fn get_user_lock_saves(env: &Env, user: &Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::UserLockSaves(user.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

// --- Internal Helper Functions ---

fn get_next_lock_id(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::NextLockId)
        .unwrap_or(1)
}

fn increment_next_lock_id(env: &Env) {
    let current_id = get_next_lock_id(env);
    env.storage()
        .persistent()
        .set(&DataKey::NextLockId, &(current_id + 1));
}

fn add_lock_to_user(env: &Env, user: &Address, lock_id: u64) {
    let mut user_locks = get_user_lock_saves(env, user);
    user_locks.push_back(lock_id);
    env.storage()
        .persistent()
        .set(&DataKey::UserLockSaves(user.clone()), &user_locks);
}

fn calculate_lock_save_yield(lock_save: &LockSave, current_time: u64) -> i128 {
    let duration_seconds = current_time.saturating_sub(lock_save.start_time);
    let duration_years = (duration_seconds as f64) / (365.25 * 24.0 * 3600.0);
    let rate_decimal = (lock_save.interest_rate as f64) / 10000.0;
    let multiplier = 1.0 + (rate_decimal * duration_years);
    (lock_save.amount as f64 * multiplier) as i128
}
