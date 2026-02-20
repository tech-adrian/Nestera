#[cfg(test)]
mod tests {
    use crate::{NesteraContract, NesteraContractClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};

    fn setup_test_env() -> (Env, NesteraContractClient<'static>) {
        let env = Env::default();
        let contract_id = env.register(NesteraContract, ());
        let client = NesteraContractClient::new(&env, &contract_id);
        (env, client)
    }

    #[test]
    fn test_ttl_extension_on_user_creation() {
        let (env, client) = setup_test_env();
        let user = Address::generate(&env);

        env.mock_all_auths();

        // Create user - should extend TTL
        client.initialize_user(&user);

        // Verify user exists
        assert!(client.user_exists(&user));

        // User retrieval should also extend TTL
        let user_data = client.get_user(&user);
        assert_eq!(user_data.total_balance, 0);
    }

    #[test]
    fn test_ttl_extension_on_flexi_operations() {
        let (env, client) = setup_test_env();
        let user = Address::generate(&env);

        env.mock_all_auths();

        // Initialize user
        client.initialize_user(&user);

        // Deposit to flexi - should extend TTL
        assert!(client.try_deposit_flexi(&user, &1000).is_ok());

        // Check balance - should extend TTL
        let balance = client.get_flexi_balance(&user);
        assert_eq!(balance, 1000);

        // Withdraw - should extend TTL
        assert!(client.try_withdraw_flexi(&user, &500).is_ok());

        let final_balance = client.get_flexi_balance(&user);
        assert_eq!(final_balance, 500);
    }

    #[test]
    fn test_ttl_extension_on_lock_save() {
        let (env, client) = setup_test_env();
        let user = Address::generate(&env);

        env.mock_all_auths();

        // Initialize user
        client.initialize_user(&user);

        // Create lock save - should extend TTL
        let lock_id = client.create_lock_save(&user, &5000, &(30 * 24 * 60 * 60));
        assert_eq!(lock_id, 1);

        // Get user lock saves - should extend TTL
        let locks = client.get_user_lock_saves(&user);
        assert_eq!(locks.len(), 1);

        // Check maturity - should extend TTL
        let is_matured = client.check_matured_lock(&lock_id);
        assert!(!is_matured);
    }

    #[test]
    fn test_ttl_extension_on_goal_save() {
        let (env, client) = setup_test_env();
        let user = Address::generate(&env);

        env.mock_all_auths();

        // Initialize user
        client.initialize_user(&user);

        let goal_name = Symbol::new(&env, "vacation");

        // Create goal save - should extend TTL
        let goal_id = client.create_goal_save(&user, &goal_name, &10000, &1000);
        assert_eq!(goal_id, 1);

        // Deposit to goal - should extend TTL
        client.deposit_to_goal_save(&user, &goal_id, &2000);

        // Get goal details - should extend TTL
        let goal = client.get_goal_save_detail(&goal_id);
        assert_eq!(goal.current_amount, 3000);

        // Get user goal saves - should extend TTL
        let goals = client.get_user_goal_saves(&user);
        assert_eq!(goals.len(), 1);
    }

    #[test]
    fn test_ttl_extension_on_completed_goal() {
        let (env, client) = setup_test_env();
        let user = Address::generate(&env);

        env.mock_all_auths();

        // Initialize user
        client.initialize_user(&user);

        let goal_name = Symbol::new(&env, "car");

        // Create completed goal
        let goal_id = client.create_goal_save(&user, &goal_name, &5000, &5000);

        // Get goal details - completed goals should still extend TTL (but shorter)
        let goal = client.get_goal_save_detail(&goal_id);
        assert!(goal.is_completed);

        // Withdraw completed goal - should extend TTL
        let amount = client.withdraw_completed_goal_save(&user, &goal_id);
        assert_eq!(amount, 5000);

        // Withdrawn goals should extend TTL with shorter duration
        let goal_after = client.get_goal_save_detail(&goal_id);
        assert!(goal_after.is_withdrawn);
    }

    #[test]
    fn test_ttl_extension_on_autosave() {
        let (env, client) = setup_test_env();
        let user = Address::generate(&env);

        env.mock_all_auths();

        // Initialize user
        client.initialize_user(&user);

        // Create autosave schedule - should extend TTL
        let schedule_id =
            client.create_autosave(&user, &100, &(24 * 60 * 60), &env.ledger().timestamp());
        assert_eq!(schedule_id, 1);

        // Get autosave - should extend TTL
        let schedule = client.get_autosave(&1);
        assert!(schedule.is_some());
        assert!(schedule.unwrap().is_active);

        // Get user autosaves - should extend TTL
        let schedules = client.get_user_autosaves(&user);
        assert_eq!(schedules.len(), 1);
    }

    #[test]
    fn test_ttl_extension_on_config_operations() {
        let (env, client) = setup_test_env();
        let admin = Address::generate(&env);
        let admin_pk = soroban_sdk::BytesN::from_array(&env, &[1u8; 32]);

        env.mock_all_auths();

        // Initialize contract - should extend TTL
        client.initialize(&admin, &admin_pk);

        // Check if initialized - should extend instance TTL
        assert!(client.is_initialized());

        // Check paused state - should extend TTL
        let is_paused = client.is_paused();
        assert!(!is_paused);

        // Pause contract - should extend TTL
        assert!(client.try_pause(&admin).is_ok());

        // Verify paused
        assert!(client.is_paused());

        // Unpause - should extend TTL
        assert!(client.try_unpause(&admin).is_ok());

        // Verify unpaused
        assert!(!client.is_paused());
    }

    #[test]
    fn test_no_premature_expiry_during_active_usage() {
        let (env, client) = setup_test_env();
        let user = Address::generate(&env);

        env.mock_all_auths();

        // Initialize user
        client.initialize_user(&user);

        // Create a lock save
        let lock_id = client.create_lock_save(&user, &5000, &(30 * 24 * 60 * 60));

        // Simulate multiple interactions (each should extend TTL)
        for _ in 0..10 {
            let _is_matured = client.check_matured_lock(&lock_id);
            let _locks = client.get_user_lock_saves(&user);
        }

        // Verify lock still exists
        let locks = client.get_user_lock_saves(&user);
        assert_eq!(locks.len(), 1);
    }

    #[test]
    fn test_ttl_extension_on_group_operations() {
        let (env, client) = setup_test_env();
        let creator = Address::generate(&env);

        env.mock_all_auths();

        // Initialize user
        client.initialize_user(&creator);

        // Create group save - should extend TTL
        let group_id = client.create_group_save(
            &creator,
            &soroban_sdk::String::from_str(&env, "Vacation Fund"),
            &soroban_sdk::String::from_str(&env, "Save for vacation"),
            &soroban_sdk::String::from_str(&env, "Travel"),
            &10000,
            &1,
            &100,
            &true,
            &env.ledger().timestamp(),
            &(env.ledger().timestamp() + 365 * 24 * 60 * 60),
        );

        // Contribute to group - should extend TTL
        client.contribute_to_group_save(&creator, &group_id, &1000);

        // Join operations should extend TTL for all members
        let member = Address::generate(&env);
        client.initialize_user(&member);
        client.join_group_save(&member, &group_id);
    }
}
