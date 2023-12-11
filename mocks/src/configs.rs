use ibc_testkit::fixtures::core::signer::dummy_bech32_account;
use sov_modules_api::Address;
use typed_builder::TypedBuilder;

/// Configuration for the `transfer` tests.
#[derive(TypedBuilder, Clone, Debug)]
pub struct TransferTestConfig {
    /// The token name on the rollup.
    pub sov_denom: String,
    /// The token address on the rollup.
    #[builder(default = None)]
    pub sov_token_address: Option<Address>,
    /// An arbitrary user address on the rollup.
    pub sov_address: Address,
    /// The token name on the Cosmos chain.
    #[builder(default = "basecoin".to_string())]
    pub cos_denom: String,
    /// An arbitrary user address on the Cosmos chain.
    #[builder(default = dummy_bech32_account())]
    pub cos_address: String,
    /// The amount to transfer.
    #[builder(default = 100)]
    pub amount: u64,
}
