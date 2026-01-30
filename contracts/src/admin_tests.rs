use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Error, InvokeError, Symbol};

use crate::{NesteraContract, NesteraContractClient, SavingsError};

fn setup() -> (Env, NesteraContractClient<'static>, Address) {
	let env = Env::default();
	let contract_id = env.register(NesteraContract, ());
	let client = NesteraContractClient::new(&env, &contract_id);
	let admin = Address::generate(&env);
	let admin_pk = BytesN::from_array(&env, &[1u8; 32]);

	env.mock_all_auths();
	client.initialize(&admin, &admin_pk);

	(env, client, admin)
}

fn assert_contract_error(err: Result<Error, InvokeError>, expected: SavingsError) {
	assert_eq!(err, Ok(Error::from_contract_error(expected as u32)));
}

fn assert_savings_error(err: Result<SavingsError, InvokeError>, expected: SavingsError) {
	assert_eq!(err, Ok(expected));
}

#[test]
fn non_admin_cannot_pause_or_unpause() {
	let (env, client, _admin) = setup();
	let non_admin = Address::generate(&env);

	env.mock_all_auths();
	assert_savings_error(
		client.try_pause(&non_admin).unwrap_err(),
		SavingsError::Unauthorized,
	);
	assert_savings_error(
		client.try_unpause(&non_admin).unwrap_err(),
		SavingsError::Unauthorized,
	);
}

#[test]
fn paused_blocks_write_paths() {
	let (env, client, admin) = setup();
	let user = Address::generate(&env);

	env.mock_all_auths();
	assert!(client.try_pause(&admin).is_ok());

	assert_savings_error(
		client.try_initialize_user(&user).unwrap_err(),
		SavingsError::ContractPaused,
	);

	assert_contract_error(
		client.try_init_user(&user).unwrap_err(),
		SavingsError::ContractPaused,
	);

	assert_contract_error(
		client
			.try_create_savings_plan(&user, &crate::storage_types::PlanType::Flexi, &100)
			.unwrap_err(),
		SavingsError::ContractPaused,
	);

	assert_savings_error(
		client.try_deposit_flexi(&user, &10).unwrap_err(),
		SavingsError::ContractPaused,
	);
	assert_savings_error(
		client.try_withdraw_flexi(&user, &5).unwrap_err(),
		SavingsError::ContractPaused,
	);

	assert_contract_error(
		client.try_create_lock_save(&user, &100, &30).unwrap_err(),
		SavingsError::ContractPaused,
	);

	assert_contract_error(
		client.try_withdraw_lock_save(&user, &1).unwrap_err(),
		SavingsError::ContractPaused,
	);

	let goal_name = Symbol::new(&env, "goal");
	assert_contract_error(
		client
			.try_create_goal_save(&user, &goal_name, &1000, &100)
			.unwrap_err(),
		SavingsError::ContractPaused,
	);

	assert_contract_error(
		client.try_deposit_to_goal_save(&user, &1, &50).unwrap_err(),
		SavingsError::ContractPaused,
	);

	assert_contract_error(
		client
			.try_withdraw_completed_goal_save(&user, &1)
			.unwrap_err(),
		SavingsError::ContractPaused,
	);

	assert_contract_error(
		client.try_break_goal_save(&user, &1).unwrap_err(),
		SavingsError::ContractPaused,
	);

	assert_savings_error(
		client
			.try_create_group_save(
				&user,
				&soroban_sdk::String::from_str(&env, "title"),
				&soroban_sdk::String::from_str(&env, "desc"),
				&soroban_sdk::String::from_str(&env, "cat"),
				&1000,
				&0,
				&10,
				&true,
				&1,
				&2,
			)
			.unwrap_err(),
		SavingsError::ContractPaused,
	);

	assert_savings_error(
		client.try_join_group_save(&user, &1).unwrap_err(),
		SavingsError::ContractPaused,
	);
	assert_savings_error(
		client.try_contribute_to_group_save(&user, &1, &10).unwrap_err(),
		SavingsError::ContractPaused,
	);
}

#[test]
fn unpause_restores_write_paths() {
	let (env, client, admin) = setup();
	let user = Address::generate(&env);

	env.mock_all_auths();
	assert!(client.try_pause(&admin).is_ok());
	assert!(client.try_unpause(&admin).is_ok());

	assert!(client.try_initialize_user(&user).is_ok());
}

