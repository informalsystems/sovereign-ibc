use std::time::Duration;

use basecoin_store::impls::InMemoryStore;
use ibc::clients::ics07_tendermint::client_type as tm_client_type;
use ibc::core::ics24_host::identifier::ClientId;
use ibc::core::ValidationContext;
use tokio::time::sleep;

use super::cosmos::helpers::dummy_signer;
use super::relayer::Relayer;
use crate::test_utils::cosmos::builder::CosmosBuilder;
use crate::test_utils::relayer::relay::MockRelayer;
use crate::test_utils::sovereign::builder::Builder;

/// Set ups a relayer between a mock sovereign chain and a mock cosmos chain
pub async fn sovereign_cosmos_setup(sov_builder: &mut Builder, with_initial_tao: bool) -> Relayer {
    let mut sov_chain = sov_builder.build();

    let sov_client_counter = sov_chain.ibc_ctx().client_counter().unwrap();

    // TODO: this should be updated when there is a client type for sovereign chains
    let sov_client_id = ClientId::new(tm_client_type(), sov_client_counter).unwrap();

    if with_initial_tao {
        sov_chain.setup_client();

        sov_chain.setup_connection();

        sov_chain.setup_channel();
    }

    let mut cos_builder = CosmosBuilder::default();

    let cos_chain = cos_builder.build_chain(InMemoryStore::default());

    let cos_client_counter = cos_chain.ibc_ctx().client_counter().unwrap();

    let cos_client_id = ClientId::new(tm_client_type(), cos_client_counter).unwrap();

    // Waits for the mock Cosmos chain to generate a few blocks before proceeding.
    sleep(Duration::from_secs(1)).await;

    MockRelayer::new(
        sov_chain.into(),
        cos_chain,
        sov_client_id,
        cos_client_id,
        dummy_signer(),
        dummy_signer(),
    )
}
