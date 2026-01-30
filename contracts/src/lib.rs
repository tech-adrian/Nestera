#![no_std]
#![allow(non_snake_case)]
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, xdr::ToXdr, Address, Bytes, BytesN,
    Env, String, Symbol, Vec,
};

mod errors;
mod flexi;
mod goal;
mod group;
mod lock;
mod storage_types;
mod users;
mod views;

pub use crate::errors::SavingsError;
pub use crate::storage_types::{
    DataKey, GoalSave, GoalSaveView, GroupSave, GroupSaveView, LockSave, LockSaveView, MintPayload,
    PlanType, SavingsPlan, User,
};

/// Custom error codes for the contract administration
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidSignature = 3,
    SignatureExpired = 4,
}

impl From<ContractError> for soroban_sdk::Error {
    fn from(e: ContractError) -> Self {
        soroban_sdk::Error::from_contract_error(e as u32)
    }
}

#[contract]
pub struct NesteraContract;

pub(crate) fn ensure_not_paused(env: &Env) -> Result<(), SavingsError> {
    let is_paused: bool = env
        .storage()
        .persistent()
        .get(&DataKey::Paused)
        .unwrap_or(false);
    if is_paused {
        Err(SavingsError::ContractPaused)
    } else {
        Ok(())
    }
}

#[contractimpl]
impl NesteraContract {
    /// Initialize a new user in the system
    pub fn init_user(env: Env, user: Address) -> User {
        ensure_not_paused(&env).unwrap_or_else(|e| panic_with_error!(&env, e));
        users::initialize_user(&env, user.clone()).unwrap_or_else(|e| panic_with_error!(&env, e));
        users::get_user(&env, &user).unwrap_or_else(|e| panic_with_error!(&env, e))
    }

    pub fn initialize(env: Env, admin: Address, admin_public_key: BytesN<32>) {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::AdminPublicKey, &admin_public_key);
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().persistent().set(&DataKey::Paused, &false);
        env.events()
            .publish((symbol_short!("init"),), admin_public_key);
    }

    pub fn verify_signature(env: Env, payload: MintPayload, signature: BytesN<64>) -> bool {
        if !env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(&env, ContractError::NotInitialized);
        }
        let current_timestamp = env.ledger().timestamp();
        let expiry_time = payload.timestamp + payload.expiry_duration;
        if current_timestamp > expiry_time {
            panic_with_error!(&env, ContractError::SignatureExpired);
        }
        let admin_public_key: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::AdminPublicKey)
            .expect("Admin PK not found");
        let payload_bytes: Bytes = payload.to_xdr(&env);
        env.crypto()
            .ed25519_verify(&admin_public_key, &payload_bytes, &signature);
        true
    }

    pub fn mint(env: Env, payload: MintPayload, signature: BytesN<64>) -> i128 {
        Self::verify_signature(env.clone(), payload.clone(), signature);
        let amount = payload.amount;
        env.events()
            .publish((symbol_short!("mint"), payload.user), amount);
        amount
    }

    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    pub fn create_savings_plan(
        env: Env,
        user: Address,
        plan_type: PlanType,
        initial_deposit: i128,
    ) -> u64 {
        ensure_not_paused(&env).unwrap_or_else(|e| panic_with_error!(&env, e));
        if !Self::is_initialized(env.clone()) {
            panic_with_error!(&env, ContractError::NotInitialized);
        }
        let mut user_data = Self::get_user(env.clone(), user.clone()).unwrap_or(User {
            total_balance: 0,
            savings_count: 0,
        });
        user_data.savings_count += 1;
        user_data.total_balance += initial_deposit;
        let plan_id = user_data.savings_count as u64;
        let new_plan = SavingsPlan {
            plan_id,
            plan_type,
            balance: initial_deposit,
            start_time: env.ledger().timestamp(),
            last_deposit: env.ledger().timestamp(),
            last_withdraw: 0,
            interest_rate: 500,
            is_completed: false,
            is_withdrawn: false,
        };
        env.storage()
            .persistent()
            .set(&DataKey::User(user.clone()), &user_data);
        env.storage()
            .persistent()
            .set(&DataKey::SavingsPlan(user.clone(), plan_id), &new_plan);
        env.events().publish(
            (Symbol::new(&env, "create_plan"), user, plan_id),
            initial_deposit,
        );
        plan_id
    }

    // --- User & Flexi Logic ---

    pub fn get_user(env: Env, user: Address) -> Result<User, SavingsError> {
        users::get_user(&env, &user)
    }

    pub fn initialize_user(env: Env, user: Address) -> Result<(), SavingsError> {
        ensure_not_paused(&env)?;
        users::initialize_user(&env, user)
    }

    pub fn user_exists(env: Env, user: Address) -> bool {
        users::user_exists(&env, &user)
    }

    pub fn deposit_flexi(env: Env, user: Address, amount: i128) -> Result<(), SavingsError> {
        ensure_not_paused(&env)?;
        flexi::flexi_deposit(env, user, amount)
    }

    pub fn withdraw_flexi(env: Env, user: Address, amount: i128) -> Result<(), SavingsError> {
        ensure_not_paused(&env)?;
        flexi::flexi_withdraw(env, user, amount)
    }

    pub fn get_flexi_balance(env: Env, user: Address) -> i128 {
        flexi::get_flexi_balance(&env, user).unwrap_or(0)
    }

    // --- Lock Save Logic ---

    pub fn create_lock_save(env: Env, user: Address, amount: i128, duration: u64) -> u64 {
        ensure_not_paused(&env).unwrap_or_else(|e| panic_with_error!(&env, e));
        user.require_auth();
        lock::create_lock_save(&env, user, amount, duration)
            .unwrap_or_else(|e| panic_with_error!(&env, e))
    }

    pub fn withdraw_lock_save(env: Env, user: Address, lock_id: u64) -> i128 {
        ensure_not_paused(&env).unwrap_or_else(|e| panic_with_error!(&env, e));
        user.require_auth();
        lock::withdraw_lock_save(&env, user, lock_id).unwrap_or_else(|e| panic_with_error!(&env, e))
    }

    pub fn check_matured_lock(env: Env, lock_id: u64) -> bool {
        lock::check_matured_lock(&env, lock_id)
    }

    pub fn get_user_lock_saves(env: Env, user: Address) -> Vec<u64> {
        lock::get_user_lock_saves(&env, &user)
    }

    // ========== Goal Save Functions ==========

    pub fn create_goal_save(
        env: Env,
        user: Address,
        goal_name: Symbol,
        target_amount: i128,
        initial_deposit: i128,
    ) -> u64 {
        ensure_not_paused(&env).unwrap_or_else(|e| panic_with_error!(&env, e));
        goal::create_goal_save(&env, user, goal_name, target_amount, initial_deposit)
            .unwrap_or_else(|e| panic_with_error!(&env, e))
    }

    pub fn deposit_to_goal_save(env: Env, user: Address, goal_id: u64, amount: i128) {
        ensure_not_paused(&env).unwrap_or_else(|e| panic_with_error!(&env, e));
        goal::deposit_to_goal_save(&env, user, goal_id, amount)
            .unwrap_or_else(|e| panic_with_error!(&env, e))
    }

    pub fn withdraw_completed_goal_save(env: Env, user: Address, goal_id: u64) -> i128 {
        ensure_not_paused(&env).unwrap_or_else(|e| panic_with_error!(&env, e));
        goal::withdraw_completed_goal_save(&env, user, goal_id)
            .unwrap_or_else(|e| panic_with_error!(&env, e))
    }

    pub fn break_goal_save(env: Env, user: Address, goal_id: u64) {
        ensure_not_paused(&env).unwrap_or_else(|e| panic_with_error!(&env, e));
        goal::break_goal_save(&env, user, goal_id).unwrap_or_else(|e| panic_with_error!(&env, e))
    }

    pub fn get_goal_save_detail(env: Env, goal_id: u64) -> GoalSave {
        goal::get_goal_save(&env, goal_id)
            .unwrap_or_else(|| panic_with_error!(&env, SavingsError::PlanNotFound))
    }

    pub fn get_user_goal_saves(env: Env, user: Address) -> Vec<u64> {
        goal::get_user_goal_saves(&env, &user)
    }

    // --- Group Save Logic ---

    pub fn create_group_save(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        category: String,
        target_amount: i128,
        contribution_type: u32,
        contribution_amount: i128,
        is_public: bool,
        start_time: u64,
        end_time: u64,
    ) -> Result<u64, SavingsError> {
        ensure_not_paused(&env)?;
        group::create_group_save(
            &env,
            creator,
            title,
            description,
            category,
            target_amount,
            contribution_type,
            contribution_amount,
            is_public,
            start_time,
            end_time,
        )
    }

    pub fn join_group_save(env: Env, user: Address, group_id: u64) -> Result<(), SavingsError> {
        ensure_not_paused(&env)?;
        group::join_group_save(&env, user, group_id)
    }

    pub fn contribute_to_group_save(
        env: Env,
        user: Address,
        group_id: u64,
        amount: i128,
    ) -> Result<(), SavingsError> {
        ensure_not_paused(&env)?;
        group::contribute_to_group_save(&env, user, group_id, amount)
    }

    // --- Admin Control Functions ---

    pub fn set_admin(
        env: Env,
        current_admin: Address,
        new_admin: Address,
    ) -> Result<(), SavingsError> {
        current_admin.require_auth();
        let stored_admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        if let Some(admin) = stored_admin {
            if admin != current_admin {
                return Err(SavingsError::Unauthorized);
            }
        }
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        env.events()
            .publish((symbol_short!("set_admin"),), new_admin);
        Ok(())
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), SavingsError> {
        admin.require_auth();
        let stored_admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);

        // Use .clone() here so 'admin' isn't moved
        if stored_admin != Some(admin.clone()) {
            return Err(SavingsError::Unauthorized);
        }

        env.storage().persistent().set(&DataKey::Paused, &true);
        env.events().publish((symbol_short!("pause"), admin), ());
        Ok(())
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), SavingsError> {
        admin.require_auth();
        let stored_admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);

        // Use .clone() here too
        if stored_admin != Some(admin.clone()) {
            return Err(SavingsError::Unauthorized);
        }

        env.storage().persistent().set(&DataKey::Paused, &false);
        env.events().publish((symbol_short!("unpause"), admin), ());
        Ok(())
    }

    // --- Remaining views and utilities ---
    pub fn get_savings_plan(env: Env, user: Address, plan_id: u64) -> Option<SavingsPlan> {
        env.storage()
            .persistent()
            .get(&DataKey::SavingsPlan(user, plan_id))
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod admin_tests;
#[cfg(test)]
mod test;
