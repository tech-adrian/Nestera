#![no_std]
#![allow(deprecated)] // Allow deprecated events::publish for now

mod storage_types;

use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, xdr::ToXdr, Address, Bytes, BytesN, Env,
    Symbol, Vec,
};
pub use storage_types::{DataKey, MintPayload, PlanType, SavingsPlan, User};

/// Custom error codes for the contract
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    /// Contract has already been initialized
    AlreadyInitialized = 1,
    /// Contract has not been initialized
    NotInitialized = 2,
    /// Invalid signature provided
    InvalidSignature = 3,
    /// Signature has expired
    SignatureExpired = 4,
}

impl From<ContractError> for soroban_sdk::Error {
    fn from(e: ContractError) -> Self {
        soroban_sdk::Error::from_contract_error(e as u32)
    }
}

#[contract]
pub struct NesteraContract;

#[contractimpl]
impl NesteraContract {
    /// Initializes the contract with the admin's Ed25519 public key.
    /// This function can only be called once.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `admin_public_key` - The 32-byte Ed25519 public key of the admin
    ///
    /// # Panics
    /// Panics if the contract has already been initialized.
    pub fn initialize(env: Env, admin_public_key: BytesN<32>) {
        // Check if already initialized
        if env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }

        // Store the admin public key
        env.storage()
            .instance()
            .set(&DataKey::AdminPublicKey, &admin_public_key);

        // Mark as initialized
        env.storage().instance().set(&DataKey::Initialized, &true);

        // Emit initialization event
        env.events()
            .publish((symbol_short!("init"),), admin_public_key);
    }

    /// Verifies that a signature is valid for the given payload.
    /// This is the core security checkpoint that validates admin approval.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `payload` - The mint payload that was signed
    /// * `signature` - The 64-byte Ed25519 signature from the admin
    ///
    /// # Panics
    /// * Panics if the contract is not initialized
    /// * Panics if the signature is invalid
    /// * Panics if the signature has expired
    pub fn verify_signature(env: Env, payload: MintPayload, signature: BytesN<64>) -> bool {
        // Ensure contract is initialized
        if !env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(&env, ContractError::NotInitialized);
        }

        // Check signature expiry using ledger timestamp
        let current_timestamp = env.ledger().timestamp();
        let expiry_time = payload.timestamp + payload.expiry_duration;

        if current_timestamp > expiry_time {
            panic_with_error!(&env, ContractError::SignatureExpired);
        }

        // Fetch admin public key from storage
        let admin_public_key: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::AdminPublicKey)
            .expect("Admin public key not found");

        // Serialize the payload to XDR bytes for verification
        // This ensures consistent serialization between off-chain signing and on-chain verification
        let payload_bytes: Bytes = payload.to_xdr(&env);

        // Verify the Ed25519 signature
        // This will panic if the signature is invalid
        env.crypto()
            .ed25519_verify(&admin_public_key, &payload_bytes, &signature);

        true
    }

    /// Mints tokens for a user after verifying the admin's signature.
    /// Users call this function themselves, paying the gas fees.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `payload` - The mint payload containing user address and amount
    /// * `signature` - The 64-byte Ed25519 signature from the admin
    ///
    /// # Returns
    /// The amount of tokens minted
    ///
    /// # Panics
    /// * Panics if signature verification fails
    /// * Panics if the signature has expired
    pub fn mint(env: Env, payload: MintPayload, signature: BytesN<64>) -> i128 {
        // Verify the signature first - this is the security checkpoint
        Self::verify_signature(env.clone(), payload.clone(), signature);

        // At this point, the signature is valid and not expired
        // The user is authorized to mint the specified amount

        let amount = payload.amount;
        let user = payload.user.clone();

        // Emit mint event
        env.events()
            .publish((symbol_short!("mint"), user.clone()), amount);

        // TODO: Implement actual token minting logic here
        // This would typically interact with a token contract
        // For now, we return the amount that would be minted

        amount
    }

    /// Returns the stored admin public key.
    /// Useful for off-chain verification and debugging.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    ///
    /// # Returns
    /// The 32-byte admin public key
    ///
    /// # Panics
    /// Panics if the contract is not initialized.
    pub fn get_admin_public_key(env: Env) -> BytesN<32> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(&env, ContractError::NotInitialized);
        }

        env.storage()
            .instance()
            .get(&DataKey::AdminPublicKey)
            .expect("Admin public key not found")
    }

    /// Checks if the contract has been initialized.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    ///
    /// # Returns
    /// `true` if initialized, `false` otherwise
    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    pub fn create_savings_plan(
        env: Env,
        user: Address,
        plan_type: PlanType,
        initial_deposit: i128,
    ) -> u64 {
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
            plan_type: plan_type.clone(),
            balance: initial_deposit,
            start_time: env.ledger().timestamp(),
            last_deposit: env.ledger().timestamp(),
            last_withdraw: 0,
            interest_rate: 500, // Default 5%
            is_completed: false,
        };

        // Store user data
        env.storage().persistent().set(&DataKey::User(user.clone()), &user_data);

        // Store plan data
        env.storage()
            .persistent()
            .set(&DataKey::SavingsPlan(user.clone(), plan_id), &new_plan);

        // Emit event
        env.events().publish(
            (Symbol::new(&env, "create_plan"), user, plan_id),
            initial_deposit,
        );

        plan_id
    }

    pub fn get_savings_plan(env: Env, user: Address, plan_id: u64) -> Option<SavingsPlan> {
        env.storage()
            .persistent()
            .get(&DataKey::SavingsPlan(user, plan_id))
    }

    pub fn get_user_savings_plans(env: Env, user: Address) -> Vec<SavingsPlan> {
        let user_data = Self::get_user(env.clone(), user.clone());
        let mut plans = Vec::new(&env);

        if let Some(data) = user_data {
            for i in 1..=data.savings_count {
                let plan_id = i as u64;
                if let Some(plan) = Self::get_savings_plan(env.clone(), user.clone(), plan_id) {
                    plans.push_back(plan);
                }
            }
        }
        plans
    }

    pub fn get_user(env: Env, user: Address) -> Option<User> {
        env.storage().persistent().get(&DataKey::User(user))
    }
}

#[cfg(test)]
mod test;
