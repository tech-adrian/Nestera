#![cfg(test)]
extern crate std;

use crate::{MintPayload, NesteraContract, NesteraContractClient, PlanType, SavingsPlan, User};
use ed25519_dalek::{Signer, SigningKey};
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{xdr::ToXdr, Address, Bytes, BytesN, Env, Vec, symbol_short};

/// Helper function to create a test environment and contract client
fn setup_test_env() -> (Env, NesteraContractClient<'static>) {
    let env = Env::default();
    let contract_id = env.register(NesteraContract, ());
    let client = NesteraContractClient::new(&env, &contract_id);
    (env, client)
}

/// Helper function to generate an Ed25519 keypair for testing
/// Returns (signing_key, public_key_bytes)
fn generate_keypair(env: &Env) -> (SigningKey, BytesN<32>) {
    // Create a deterministic signing key for testing
    let secret_bytes: [u8; 32] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    ];
    let signing_key = SigningKey::from_bytes(&secret_bytes);

    // Get the public key bytes
    let public_key = signing_key.verifying_key();
    let public_key_bytes: BytesN<32> = BytesN::from_array(env, &public_key.to_bytes());

    (signing_key, public_key_bytes)
}

/// Generate a second keypair (attacker) for testing wrong signer scenarios
fn generate_attacker_keypair(env: &Env) -> (SigningKey, BytesN<32>) {
    let secret_bytes: [u8; 32] = [
        99, 98, 97, 96, 95, 94, 93, 92, 91, 90, 89, 88, 87, 86, 85, 84, 83, 82, 81, 80, 79, 78, 77,
        76, 75, 74, 73, 72, 71, 70, 69, 68,
    ];
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let public_key = signing_key.verifying_key();
    let public_key_bytes: BytesN<32> = BytesN::from_array(env, &public_key.to_bytes());

    (signing_key, public_key_bytes)
}

/// Helper to sign a payload with the admin's secret key
fn sign_payload(env: &Env, signing_key: &SigningKey, payload: &MintPayload) -> BytesN<64> {
    // Serialize payload to XDR (same as contract does)
    let payload_bytes: Bytes = payload.to_xdr(env);

    // Convert Bytes to Vec<u8> for signing
    let len = payload_bytes.len() as usize;
    let mut payload_slice: std::vec::Vec<u8> = std::vec![0u8; len];
    payload_bytes.copy_into_slice(&mut payload_slice);

    // Sign with ed25519_dalek
    let signature = signing_key.sign(&payload_slice);

    // Convert signature to BytesN<64>
    BytesN::from_array(env, &signature.to_bytes())
}

/// Helper to set the ledger timestamp
fn set_ledger_timestamp(env: &Env, timestamp: u64) {
    env.ledger().set(LedgerInfo {
        timestamp,
        protocol_version: 23,
        sequence_number: 100,
        network_id: [0u8; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });
}

// =============================================================================
// Initialization Tests
// =============================================================================

#[test]
fn test_initialize_success() {
    let (env, client) = setup_test_env();
    let (_, admin_public_key) = generate_keypair(&env);

    // Should not be initialized yet
    assert!(!client.is_initialized());

    // Initialize the contract
    client.initialize(&admin_public_key);

    // Should be initialized now
    assert!(client.is_initialized());

    // Verify the stored public key matches
    let stored_key = client.get_admin_public_key();
    assert_eq!(stored_key, admin_public_key);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_initialize_already_initialized() {
    let (env, client) = setup_test_env();
    let (_, admin_public_key) = generate_keypair(&env);

    // Initialize once
    client.initialize(&admin_public_key);

    // Try to initialize again - should panic with AlreadyInitialized (error code 1)
    client.initialize(&admin_public_key);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_get_admin_public_key_not_initialized() {
    let (_, client) = setup_test_env();

    // Should panic with NotInitialized (error code 2)
    client.get_admin_public_key();
}

// =============================================================================
// Signature Verification Tests
// =============================================================================

#[test]
fn test_verify_signature_success() {
    let (env, client) = setup_test_env();
    let (signing_key, admin_public_key) = generate_keypair(&env);

    // Initialize with admin public key
    client.initialize(&admin_public_key);

    // Set ledger timestamp
    let current_time = 1000u64;
    set_ledger_timestamp(&env, current_time);

    // Create a mint payload
    let user = Address::generate(&env);
    let payload = MintPayload {
        user: user.clone(),
        amount: 100_i128,
        timestamp: current_time,
        expiry_duration: 3600, // 1 hour validity
    };

    // Sign the payload with admin's secret key
    let signature = sign_payload(&env, &signing_key, &payload);

    // Verify should succeed and return true
    assert!(client.verify_signature(&payload, &signature));
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_verify_signature_not_initialized() {
    let (env, client) = setup_test_env();
    let (signing_key, _) = generate_keypair(&env);

    let user = Address::generate(&env);
    let payload = MintPayload {
        user,
        amount: 100_i128,
        timestamp: 1000,
        expiry_duration: 3600,
    };

    let signature = sign_payload(&env, &signing_key, &payload);

    // Should panic because contract is not initialized
    client.verify_signature(&payload, &signature);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_verify_signature_expired() {
    let (env, client) = setup_test_env();
    let (signing_key, admin_public_key) = generate_keypair(&env);

    client.initialize(&admin_public_key);

    // Create a payload that was signed in the past
    let user = Address::generate(&env);
    let payload = MintPayload {
        user,
        amount: 100_i128,
        timestamp: 1000,
        expiry_duration: 3600, // Expires at 4600
    };

    let signature = sign_payload(&env, &signing_key, &payload);

    // Set ledger timestamp to after expiry
    set_ledger_timestamp(&env, 5000);

    // Should panic with SignatureExpired (error code 4)
    client.verify_signature(&payload, &signature);
}

#[test]
#[should_panic]
fn test_verify_signature_invalid_signature() {
    let (env, client) = setup_test_env();
    let (signing_key, admin_public_key) = generate_keypair(&env);

    client.initialize(&admin_public_key);

    let current_time = 1000u64;
    set_ledger_timestamp(&env, current_time);

    let user = Address::generate(&env);
    let payload = MintPayload {
        user,
        amount: 100_i128,
        timestamp: current_time,
        expiry_duration: 3600,
    };

    // Sign with admin key
    let signature = sign_payload(&env, &signing_key, &payload);

    // Modify the payload after signing (tamper with it)
    let tampered_payload = MintPayload {
        user: Address::generate(&env), // Different user!
        amount: 100_i128,
        timestamp: current_time,
        expiry_duration: 3600,
    };

    // Should panic because signature doesn't match tampered payload
    client.verify_signature(&tampered_payload, &signature);
}

#[test]
#[should_panic]
fn test_verify_signature_wrong_signer() {
    let (env, client) = setup_test_env();
    let (_, admin_public_key) = generate_keypair(&env);
    let (attacker_signing_key, _) = generate_attacker_keypair(&env);

    client.initialize(&admin_public_key);

    let current_time = 1000u64;
    set_ledger_timestamp(&env, current_time);

    let user = Address::generate(&env);
    let payload = MintPayload {
        user,
        amount: 100_i128,
        timestamp: current_time,
        expiry_duration: 3600,
    };

    // Sign with attacker's key instead of admin's key
    let bad_signature = sign_payload(&env, &attacker_signing_key, &payload);

    // Should panic because signature is from wrong key
    client.verify_signature(&payload, &bad_signature);
}

// =============================================================================
// Mint Tests
// =============================================================================

#[test]
fn test_mint_success() {
    let (env, client) = setup_test_env();
    let (signing_key, admin_public_key) = generate_keypair(&env);

    client.initialize(&admin_public_key);

    let current_time = 1000u64;
    set_ledger_timestamp(&env, current_time);

    let user = Address::generate(&env);
    let mint_amount = 500_i128;

    let payload = MintPayload {
        user: user.clone(),
        amount: mint_amount,
        timestamp: current_time,
        expiry_duration: 3600,
    };

    let signature = sign_payload(&env, &signing_key, &payload);

    // Mint should succeed and return the amount
    let result = client.mint(&payload, &signature);
    assert_eq!(result, mint_amount);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_mint_expired_signature() {
    let (env, client) = setup_test_env();
    let (signing_key, admin_public_key) = generate_keypair(&env);

    client.initialize(&admin_public_key);

    let user = Address::generate(&env);
    let payload = MintPayload {
        user,
        amount: 500_i128,
        timestamp: 1000,
        expiry_duration: 3600,
    };

    let signature = sign_payload(&env, &signing_key, &payload);

    // Set time way past expiry
    set_ledger_timestamp(&env, 10000);

    // Should panic with SignatureExpired
    client.mint(&payload, &signature);
}

#[test]
#[should_panic]
fn test_mint_tampered_amount() {
    let (env, client) = setup_test_env();
    let (signing_key, admin_public_key) = generate_keypair(&env);

    client.initialize(&admin_public_key);

    let current_time = 1000u64;
    set_ledger_timestamp(&env, current_time);

    let user = Address::generate(&env);

    // Admin signs for 100 tokens
    let payload = MintPayload {
        user: user.clone(),
        amount: 100_i128,
        timestamp: current_time,
        expiry_duration: 3600,
    };

    let signature = sign_payload(&env, &signing_key, &payload);

    // User tries to claim 1000 tokens instead
    let tampered_payload = MintPayload {
        user,
        amount: 1000_i128, // Tampered!
        timestamp: current_time,
        expiry_duration: 3600,
    };

    // Should panic because signature doesn't match
    client.mint(&tampered_payload, &signature);
}

#[test]
fn test_mint_at_expiry_boundary() {
    let (env, client) = setup_test_env();
    let (signing_key, admin_public_key) = generate_keypair(&env);

    client.initialize(&admin_public_key);

    let sign_time = 1000u64;
    let expiry_duration = 3600u64;

    let user = Address::generate(&env);
    let payload = MintPayload {
        user,
        amount: 100_i128,
        timestamp: sign_time,
        expiry_duration,
    };

    let signature = sign_payload(&env, &signing_key, &payload);

    // Set time exactly at expiry boundary (should still work)
    set_ledger_timestamp(&env, sign_time + expiry_duration);

    // Should succeed - we're exactly at the expiry time, not past it
    let result = client.mint(&payload, &signature);
    assert_eq!(result, 100_i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_mint_one_second_after_expiry() {
    let (env, client) = setup_test_env();
    let (signing_key, admin_public_key) = generate_keypair(&env);

    client.initialize(&admin_public_key);

    let sign_time = 1000u64;
    let expiry_duration = 3600u64;

    let user = Address::generate(&env);
    let payload = MintPayload {
        user,
        amount: 100_i128,
        timestamp: sign_time,
        expiry_duration,
    };

    let signature = sign_payload(&env, &signing_key, &payload);

    // Set time one second after expiry
    set_ledger_timestamp(&env, sign_time + expiry_duration + 1);

    // Should fail - we're past the expiry time
    client.mint(&payload, &signature);
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_mint_zero_amount() {
    let (env, client) = setup_test_env();
    let (signing_key, admin_public_key) = generate_keypair(&env);

    client.initialize(&admin_public_key);

    let current_time = 1000u64;
    set_ledger_timestamp(&env, current_time);

    let user = Address::generate(&env);
    let payload = MintPayload {
        user,
        amount: 0_i128,
        timestamp: current_time,
        expiry_duration: 3600,
    };

    let signature = sign_payload(&env, &signing_key, &payload);

    // Zero amount should still work (signature is valid)
    let result = client.mint(&payload, &signature);
    assert_eq!(result, 0_i128);
}

#[test]
fn test_multiple_mints_same_user() {
    let (env, client) = setup_test_env();
    let (signing_key, admin_public_key) = generate_keypair(&env);

    client.initialize(&admin_public_key);

    let current_time = 1000u64;
    set_ledger_timestamp(&env, current_time);

    let user = Address::generate(&env);

    // First mint
    let payload1 = MintPayload {
        user: user.clone(),
        amount: 100_i128,
        timestamp: current_time,
        expiry_duration: 3600,
    };
    let signature1 = sign_payload(&env, &signing_key, &payload1);
    let result1 = client.mint(&payload1, &signature1);
    assert_eq!(result1, 100_i128);

    // Second mint with different amount
    let payload2 = MintPayload {
        user: user.clone(),
        amount: 200_i128,
        timestamp: current_time + 1, // Different timestamp makes it a unique payload
        expiry_duration: 3600,
    };
    let signature2 = sign_payload(&env, &signing_key, &payload2);
    let result2 = client.mint(&payload2, &signature2);
    assert_eq!(result2, 200_i128);
}

// =============================================================================
// Savings Plan Tests
// =============================================================================

#[test]
fn test_user_instantiation() {
    let user = User {
        total_balance: 1_000_000,
        savings_count: 3,
    };
    
    assert_eq!(user.total_balance, 1_000_000);
    assert_eq!(user.savings_count, 3);
}

#[test]
fn test_flexi_savings_plan() {
    let plan = SavingsPlan {
        plan_id: 1,
        plan_type: PlanType::Flexi,
        balance: 500_000,
        start_time: 1000000,
        last_deposit: 1000100,
        last_withdraw: 0,
        interest_rate: 500, // 5.00% APY
        is_completed: false,
    };
    
    assert_eq!(plan.plan_id, 1);
    assert_eq!(plan.plan_type, PlanType::Flexi);
    assert_eq!(plan.balance, 500_000);
    assert!(!plan.is_completed);
}

#[test]
fn test_lock_savings_plan() {
    let locked_until = 2000000;
    let plan = SavingsPlan {
        plan_id: 2,
        plan_type: PlanType::Lock(locked_until),
        balance: 1_000_000,
        start_time: 1000000,
        last_deposit: 1000000,
        last_withdraw: 0,
        interest_rate: 800,
        is_completed: false,
    };
    
    assert_eq!(plan.plan_id, 2);
    match plan.plan_type {
        PlanType::Lock(until) => assert_eq!(until, locked_until),
        _ => panic!("Expected Lock plan type"),
    }
}

#[test]
fn test_goal_savings_plan() {
    let plan = SavingsPlan {
        plan_id: 3,
        plan_type: PlanType::Goal(
            symbol_short!("education"),
            5_000_000,
            1u32, // e.g. 1 = weekly
        ),
        balance: 2_000_000,
        start_time: 1000000,
        last_deposit: 1500000,
        last_withdraw: 0,
        interest_rate: 600,
        is_completed: false,
    };
    
    assert_eq!(plan.plan_id, 3);
    match plan.plan_type {
        PlanType::Goal(category, target_amount, contribution_type) => {
            assert_eq!(category, symbol_short!("education"));
            assert_eq!(target_amount, 5_000_000);
            assert_eq!(contribution_type, 1u32);
        },
        _ => panic!("Expected Goal plan type"),
    }
}

#[test]
fn test_group_savings_plan() {
    let plan = SavingsPlan {
        plan_id: 4,
        plan_type: PlanType::Group(
            101,
            true,
            2u32,
            10_000_000
        ),
        balance: 3_000_000,
        start_time: 1000000,
        last_deposit: 1600000,
        last_withdraw: 0,
        interest_rate: 700,
        is_completed: false,
    };
    
    assert_eq!(plan.plan_id, 4);
    match plan.plan_type {
        PlanType::Group(group_id, is_public, contribution_type, target_amount) => {
            assert_eq!(group_id, 101);
            assert!(is_public);
            assert_eq!(contribution_type, 2u32);
            assert_eq!(target_amount, 10_000_000);
        },
        _ => panic!("Expected Group plan type"),
    }
}

#[test]
fn test_create_savings_plan() {
    let (env, client) = setup_test_env();
    let (_, admin_public_key) = generate_keypair(&env);
    
    client.initialize(&admin_public_key);
    
    let user = Address::generate(&env);
    let plan_type = PlanType::Flexi;
    let initial_deposit = 1000_i128;
    
    let plan_id = client.create_savings_plan(&user, &plan_type, &initial_deposit);
    assert_eq!(plan_id, 1);
    
    let plan = client.get_savings_plan(&user, &plan_id).unwrap();
    assert_eq!(plan.plan_id, plan_id);
    assert_eq!(plan.plan_type, plan_type);
    assert_eq!(plan.balance, initial_deposit);
}

#[test]
fn test_get_user_savings_plans() {
    let (env, client) = setup_test_env();
    let (_, admin_public_key) = generate_keypair(&env);
    
    client.initialize(&admin_public_key);
    
    let user = Address::generate(&env);
    
    // Create multiple plans
    let plan1_id = client.create_savings_plan(&user, &PlanType::Flexi, &1000_i128);
    let plan2_id = client.create_savings_plan(&user, &PlanType::Lock(2000000), &2000_i128);
    
    let plans = client.get_user_savings_plans(&user);
    assert_eq!(plans.len(), 2);
    
    // Verify plans are returned correctly
    let mut plan_ids = std::vec::Vec::new();
    for p in plans.iter() {
        plan_ids.push(p.plan_id);
    }
    assert!(plan_ids.contains(&plan1_id));
    assert!(plan_ids.contains(&plan2_id));
}

#[test]
fn test_get_user() {
    let (env, client) = setup_test_env();
    let (_, admin_public_key) = generate_keypair(&env);
    
    client.initialize(&admin_public_key);
    
    let user = Address::generate(&env);
    
    // User should not exist initially
    assert!(client.get_user(&user).is_none());
    
    // Create a savings plan
    client.create_savings_plan(&user, &PlanType::Flexi, &1000_i128);
    
    // User should now exist
    let user_data = client.get_user(&user).unwrap();
    assert_eq!(user_data.total_balance, 1000_i128);
    assert_eq!(user_data.savings_count, 1);
}
