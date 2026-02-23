#[cfg(test)]
mod governance_tests {
    use crate::governance::VotingConfig;
    use crate::rewards::storage_types::RewardsConfig;
    use crate::{NesteraContract, NesteraContractClient, PlanType};
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

    fn setup_contract() -> (Env, NesteraContractClient<'static>, Address) {
        let env = Env::default();
        let contract_id = env.register(NesteraContract, ());
        let client = NesteraContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let admin_pk = BytesN::from_array(&env, &[1u8; 32]);

        env.mock_all_auths();
        client.initialize(&admin, &admin_pk);

        let config = RewardsConfig {
            points_per_token: 10,
            streak_bonus_bps: 0,
            long_lock_bonus_bps: 0,
            goal_completion_bonus: 0,
            enabled: true,
            min_deposit_for_rewards: 0,
            action_cooldown_seconds: 0,
            max_daily_points: 1_000_000,
            max_streak_multiplier: 10_000,
        };
        let _ = client.initialize_rewards_config(&config);

        (env, client, admin)
    }

    #[test]
    fn test_voting_power_zero_for_new_user() {
        let (env, client, _) = setup_contract();
        let user = Address::generate(&env);

        let power = client.get_voting_power(&user);
        assert_eq!(power, 0);
    }

    #[test]
    fn test_voting_power_increases_with_deposits() {
        let (env, client, _) = setup_contract();
        let user = Address::generate(&env);
        env.mock_all_auths();

        client.initialize_user(&user);
        let _ = client.create_savings_plan(&user, &PlanType::Flexi, &1000);

        let power = client.get_voting_power(&user);
        assert_eq!(power, 1000);
    }

    #[test]
    fn test_voting_power_accumulates_across_deposits() {
        let (env, client, _) = setup_contract();
        let user = Address::generate(&env);
        env.mock_all_auths();

        client.initialize_user(&user);
        let _ = client.create_savings_plan(&user, &PlanType::Flexi, &1000);
        let _ = client.create_savings_plan(&user, &PlanType::Flexi, &500);

        let power = client.get_voting_power(&user);
        assert_eq!(power, 1500);
    }

    #[test]
    fn test_cast_vote_requires_voting_power() {
        let (env, client, _) = setup_contract();
        let user = Address::generate(&env);
        env.mock_all_auths();

        client.initialize_user(&user);

        let result = client.try_vote(&1, &1, &user);
        assert!(result.is_err());
    }

    #[test]
    fn test_cast_vote_succeeds_with_voting_power() {
        let (env, client, _) = setup_contract();
        let user = Address::generate(&env);
        env.mock_all_auths();

        client.initialize_user(&user);
        let _ = client.create_savings_plan(&user, &PlanType::Flexi, &1000);

        let result = client.try_vote(&1, &1, &user);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_voting_config() {
        let (env, client, admin) = setup_contract();
        env.mock_all_auths();

        let result = client.try_init_voting_config(&admin, &5000, &604800, &86400, &100, &10_000);
        assert!(result.is_ok());

        let config = client.try_get_voting_config().unwrap().unwrap();
        assert_eq!(config.quorum, 5000);
        assert_eq!(config.voting_period, 604800);
        assert_eq!(config.timelock_duration, 86400);
    }

    #[test]
    fn test_create_proposal() {
        let (env, client, admin) = setup_contract();
        env.mock_all_auths();

        let _ = client.init_voting_config(&admin, &5000, &604800, &86400, &100, &10_000);

        let creator = Address::generate(&env);
        let description = String::from_str(&env, "Test proposal");
        let proposal_id = client
            .try_create_proposal(&creator, &description)
            .unwrap()
            .unwrap();

        assert_eq!(proposal_id, 1);
    }

    #[test]
    fn test_get_proposal() {
        let (env, client, admin) = setup_contract();
        env.mock_all_auths();

        let _ = client.init_voting_config(&admin, &5000, &604800, &86400, &100, &10_000);

        let creator = Address::generate(&env);
        let description = String::from_str(&env, "Test proposal");
        let proposal_id = client
            .try_create_proposal(&creator, &description)
            .unwrap()
            .unwrap();

        let proposal = client.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.id, 1);
        assert_eq!(proposal.creator, creator);
        assert_eq!(proposal.executed, false);
        assert_eq!(proposal.for_votes, 0);
        assert_eq!(proposal.against_votes, 0);
    }

    #[test]
    fn test_list_proposals() {
        let (env, client, admin) = setup_contract();
        env.mock_all_auths();

        let _ = client.init_voting_config(&admin, &5000, &604800, &86400, &100, &10_000);

        let creator = Address::generate(&env);
        let desc1 = String::from_str(&env, "Proposal 1");
        let desc2 = String::from_str(&env, "Proposal 2");

        let _ = client.try_create_proposal(&creator, &desc1);
        let _ = client.try_create_proposal(&creator, &desc2);

        let proposals = client.list_proposals();
        assert_eq!(proposals.len(), 2);
        assert_eq!(proposals.get(0).unwrap(), 1);
        assert_eq!(proposals.get(1).unwrap(), 2);
    }

    #[test]
    fn test_proposal_stored_correctly() {
        let (env, client, admin) = setup_contract();
        env.mock_all_auths();

        let _ = client.init_voting_config(&admin, &5000, &604800, &86400, &100, &10_000);

        let creator = Address::generate(&env);
        let description = String::from_str(&env, "Store test");
        let proposal_id = client
            .try_create_proposal(&creator, &description)
            .unwrap()
            .unwrap();

        let proposal = client.get_proposal(&proposal_id).unwrap();
        let now = env.ledger().timestamp();

        assert_eq!(proposal.description, description);
        assert_eq!(proposal.start_time, now);
        assert_eq!(proposal.end_time, now + 604800);
    }
}
