use sov_bank::{BankConfig, TokenConfig};
use sov_modules_api::utils::generate_address as gen_address_generic;
use sov_modules_api::Context;

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
        token_name: "InitialToken".to_owned(),
        address_and_balances,
        authorized_minters: vec![],
        salt: 5,
    };

    BankConfig {
        tokens: vec![token_config],
    }
}
