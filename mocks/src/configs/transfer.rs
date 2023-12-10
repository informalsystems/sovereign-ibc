use ibc_testkit::fixtures::core::signer::dummy_bech32_account;
use sov_modules_api::Address;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder, Clone, Debug)]
pub struct TransferTestConfig {
    pub denom_on_sov: String,
    #[builder(default = None)]
    pub token_address: Option<Address>,
    #[builder(default = "basecoin".to_string())]
    pub denom_on_cos: String,
    pub address_on_sov: Address,
    #[builder(default = dummy_bech32_account())]
    pub address_on_cos: String,
    #[builder(default = 100)]
    pub amount: u64,
}
