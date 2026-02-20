use soroban_sdk::{Address, Env};

use crate::storage_types::{DataKey, GoalSave, LockSave, SavingsPlan};

// TTL Constants (in ledgers)
// Assuming ~5 seconds per ledger:
// - 1 day = 17,280 ledgers
// - 30 days = 518,400 ledgers
// - 90 days = 1,555,200 ledgers

/// Minimum threshold before TTL extension is triggered
pub const LOW_THRESHOLD: u32 = 518_400; // ~30 days

/// Maximum threshold - records near expiry should be extended
pub const HIGH_THRESHOLD: u32 = 1_036_800; // ~60 days

/// Duration to extend TTL to when triggered
pub const EXTEND_TO: u32 = 3_110_400; // ~180 days (6 months)

/// Shorter extension for completed/archived plans
pub const EXTEND_ARCHIVED: u32 = 518_400; // ~30 days

/// Extends the instance storage TTL
/// Used for contract-level configuration that should persist long-term
pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LOW_THRESHOLD, EXTEND_TO);
}

/// Extends TTL for user-related storage entries
/// Includes: User data, FlexiBalance, TotalBalance
pub fn extend_user_ttl(env: &Env, user: &Address) {
    let user_key = DataKey::User(user.clone());
    let flexi_key = DataKey::FlexiBalance(user.clone());
    let total_key = DataKey::TotalBalance(user.clone());

    // Only extend TTL if the key exists
    if env.storage().persistent().has(&user_key) {
        env.storage()
            .persistent()
            .extend_ttl(&user_key, LOW_THRESHOLD, EXTEND_TO);
    }

    if env.storage().persistent().has(&flexi_key) {
        env.storage()
            .persistent()
            .extend_ttl(&flexi_key, LOW_THRESHOLD, EXTEND_TO);
    }

    if env.storage().persistent().has(&total_key) {
        env.storage()
            .persistent()
            .extend_ttl(&total_key, LOW_THRESHOLD, EXTEND_TO);
    }
}

/// Extends TTL for a savings plan
/// Only extends if the plan is active (not completed/withdrawn)
pub fn extend_plan_ttl(env: &Env, plan_key: &DataKey) {
    // Check if the plan should be extended
    if should_extend_plan(env, plan_key) {
        env.storage()
            .persistent()
            .extend_ttl(plan_key, LOW_THRESHOLD, EXTEND_TO);
    } else {
        // For completed/archived plans, use shorter extension
        env.storage()
            .persistent()
            .extend_ttl(plan_key, LOW_THRESHOLD, EXTEND_ARCHIVED);
    }
}

/// Extends TTL for a Lock Save plan
pub fn extend_lock_ttl(env: &Env, lock_id: u64) {
    let lock_key = DataKey::LockSave(lock_id);

    if let Some(lock_save) = env
        .storage()
        .persistent()
        .get::<DataKey, LockSave>(&lock_key)
    {
        if lock_save.is_withdrawn {
            // Already withdrawn - use shorter extension
            env.storage()
                .persistent()
                .extend_ttl(&lock_key, LOW_THRESHOLD, EXTEND_ARCHIVED);
        } else {
            // Active plan - full extension
            env.storage()
                .persistent()
                .extend_ttl(&lock_key, LOW_THRESHOLD, EXTEND_TO);
        }
    }
}

/// Extends TTL for a Goal Save plan
pub fn extend_goal_ttl(env: &Env, goal_id: u64) {
    let goal_key = DataKey::GoalSave(goal_id);

    if let Some(goal_save) = env
        .storage()
        .persistent()
        .get::<DataKey, GoalSave>(&goal_key)
    {
        if goal_save.is_completed || goal_save.is_withdrawn {
            // Completed/withdrawn - use shorter extension
            env.storage()
                .persistent()
                .extend_ttl(&goal_key, LOW_THRESHOLD, EXTEND_ARCHIVED);
        } else {
            // Active plan - full extension
            env.storage()
                .persistent()
                .extend_ttl(&goal_key, LOW_THRESHOLD, EXTEND_TO);
        }
    }
}

/// Extends TTL for a Group Save plan
pub fn extend_group_ttl(env: &Env, group_id: u64) {
    let group_key = DataKey::GroupSave(group_id);
    let members_key = DataKey::GroupMembers(group_id);

    if env.storage().persistent().has(&group_key) {
        env.storage()
            .persistent()
            .extend_ttl(&group_key, LOW_THRESHOLD, EXTEND_TO);
    }

    if env.storage().persistent().has(&members_key) {
        env.storage()
            .persistent()
            .extend_ttl(&members_key, LOW_THRESHOLD, EXTEND_TO);
    }
}

/// Extends TTL for user's list of plans (Lock/Goal/Group/AutoSave)
pub fn extend_user_plan_list_ttl(env: &Env, list_key: &DataKey) {
    // Only extend TTL if the key exists
    if env.storage().persistent().has(list_key) {
        env.storage()
            .persistent()
            .extend_ttl(list_key, LOW_THRESHOLD, EXTEND_TO);
    }
}

/// Extends TTL for an AutoSave schedule
pub fn extend_autosave_ttl(env: &Env, schedule_id: u64) {
    let schedule_key = DataKey::AutoSave(schedule_id);
    // Only extend TTL if the key exists
    if env.storage().persistent().has(&schedule_key) {
        env.storage()
            .persistent()
            .extend_ttl(&schedule_key, LOW_THRESHOLD, EXTEND_TO);
    }
}

/// Extends TTL for configuration entries (rates, fees, etc.)
pub fn extend_config_ttl(env: &Env, config_key: &DataKey) {
    // Only extend TTL if the key exists
    if env.storage().persistent().has(config_key) {
        env.storage()
            .persistent()
            .extend_ttl(config_key, LOW_THRESHOLD, EXTEND_TO);
    }
}

/// Extends TTL for next ID counters
pub fn extend_counter_ttl(env: &Env, counter_key: &DataKey) {
    // Only extend TTL if the key exists
    if env.storage().persistent().has(counter_key) {
        env.storage()
            .persistent()
            .extend_ttl(counter_key, LOW_THRESHOLD, EXTEND_TO);
    }
}

// ========== Helper Functions ==========

/// Determines if a plan should receive full TTL extension
/// Returns false for completed/withdrawn plans
fn should_extend_plan(env: &Env, plan_key: &DataKey) -> bool {
    if let Some(plan) = env
        .storage()
        .persistent()
        .get::<DataKey, SavingsPlan>(plan_key)
    {
        // Don't extend completed or withdrawn plans
        return !plan.is_completed && !plan.is_withdrawn;
    }
    // If plan doesn't exist or can't be read, default to extending
    true
}
