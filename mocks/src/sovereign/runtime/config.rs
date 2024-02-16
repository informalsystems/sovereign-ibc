use std::time::{SystemTime, UNIX_EPOCH};

use sov_bank::{BankConfig, TokenConfig};
use sov_chain_state::ChainStateConfig;
use sov_ibc::ExampleModuleConfig;
use sov_ibc_transfer::TransferConfig;
use sov_modules_api::utils::generate_address as gen_address_generic;
use sov_modules_api::{Context, GasUnit};
use sov_rollup_interface::da::Time;

// The default initial slot height.
pub const DEFAULT_INIT_HEIGHT: u64 = 1;

// The default initial balance for each address.
pub const DEFAULT_INIT_BALANCE: u64 = 1000;

// The default number of addresses.
pub const DEFAULT_ADDRESS_COUNT: u64 = 3;

// The default token name.
pub const DEFAULT_TOKEN_NAME: &str = "sov-demo-token";

#[derive(Clone, Debug)]
pub struct RollupGenesisConfig<C: Context> {
    pub chain_state_config: ChainStateConfig<C>,
    pub bank_config: BankConfig<C>,
    pub ibc_config: ExampleModuleConfig,
    pub ibc_transfer_config: TransferConfig,
}

impl<C: Context> RollupGenesisConfig<C> {
    pub fn new(
        chain_state_config: ChainStateConfig<C>,
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

impl<C: Context> Default for RollupGenesisConfig<C> {
    fn default() -> Self {
        let chain_state_config = create_chain_state_config();

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
pub fn create_chain_state_config<C: Context>() -> ChainStateConfig<C> {
    let now = SystemTime::now();

    let seconds = now
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    ChainStateConfig {
        current_time: Time::from_secs(seconds.try_into().unwrap()),
        gas_price_blocks_depth: 10,
        gas_price_maximum_elasticity: 1,
        initial_gas_price: GasUnit::ZEROED,
        minimum_gas_price: GasUnit::ZEROED,
    }
}

/// Creates a bank configuration with the given number of addresses and initial balance
pub fn create_bank_config<C: Context>(addresses_count: u64, initial_balance: u64) -> BankConfig<C> {
    let address_and_balances: Vec<_> = (0..addresses_count)
        .map(|i| {
            let key = format!("key_{}", i);
            let addr = gen_address_generic::<C>(&key);
            (addr, initial_balance)
        })
        .collect();

    let genuine_token_config = TokenConfig {
        token_name: DEFAULT_TOKEN_NAME.to_owned(),
        address_and_balances: address_and_balances.clone(),
        authorized_minters: vec![],
        salt: 0,
    };

    let forged_token_config = TokenConfig {
        token_name: DEFAULT_TOKEN_NAME.to_owned(),
        address_and_balances: address_and_balances.clone(),
        authorized_minters: vec![],
        salt: 5,
    };

    BankConfig {
        tokens: vec![genuine_token_config, forged_token_config],
    }
}
