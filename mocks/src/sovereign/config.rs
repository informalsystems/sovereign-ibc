use std::time::{SystemTime, UNIX_EPOCH};

use sov_bank::{BankConfig, TokenConfig};
use sov_chain_state::ChainStateConfig;
use sov_ibc::ExampleModuleConfig;
use sov_ibc_transfer::TransferConfig;
use sov_modules_api::utils::generate_address as gen_address_generic;
use sov_modules_api::Context;
use sov_rollup_interface::da::Time;

// The default initial slot height.
const DEFAULT_INIT_HEIGHT: u64 = 10;

// The default initial balance for each address.
const DEFAULT_INIT_BALANCE: u64 = 1000;

// The default number of addresses.
const DEFAULT_ADDRESS_COUNT: u64 = 3;

// The default token name.
const DEFAULT_TOKEN_NAME: &str = "sov-demo-token";

pub struct TestConfig<C: Context> {
    pub chain_state_config: ChainStateConfig,
    pub bank_config: BankConfig<C>,
    pub ibc_config: ExampleModuleConfig,
    pub ibc_transfer_config: TransferConfig,
}

impl<C: Context> TestConfig<C> {
    pub fn new(
        chain_state_config: ChainStateConfig,
        bank_config: BankConfig<C>,
        ibc_config: ExampleModuleConfig,
        ibc_transfer_config: TransferConfig,
    ) -> Self {
        Self {
            chain_state_config,
            bank_config,
            ibc_config,
            ibc_transfer_config,
        }
    }
}

impl<C: Context> Default for TestConfig<C> {
    fn default() -> Self {
        let chain_state_config = create_chain_state_config(DEFAULT_INIT_HEIGHT);

        let bank_config = create_bank_config(DEFAULT_ADDRESS_COUNT, DEFAULT_INIT_BALANCE);

        let ibc_config = ExampleModuleConfig {};

        let ibc_transfer_config = TransferConfig {};

        Self {
            chain_state_config,
            bank_config,
            ibc_config,
            ibc_transfer_config,
        }
    }
}

/// Creates a chain state configuration with the given initial slot height
pub fn create_chain_state_config(initial_slot_height: u64) -> ChainStateConfig {
    let now = SystemTime::now();

    let seconds = now
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    ChainStateConfig {
        initial_slot_height,
        current_time: Time::from_secs(seconds.try_into().unwrap()),
    }
}

/// Creates a bank configuration with the given number of addresses and initial balance
pub fn create_bank_config<C: Context>(addresses_count: u64, initial_balance: u64) -> BankConfig<C> {
    let address_and_balances = (0..addresses_count)
        .map(|i| {
            let key = format!("key_{}", i);
            let addr = gen_address_generic::<C>(&key);
            (addr, initial_balance)
        })
        .collect();

    let token_config = TokenConfig {
        token_name: DEFAULT_TOKEN_NAME.to_owned(),
        address_and_balances,
        authorized_minters: vec![],
        salt: 5,
    };

    BankConfig {
        tokens: vec![token_config],
    }
}
