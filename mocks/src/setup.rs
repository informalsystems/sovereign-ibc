use std::time::Duration;

use basecoin_store::impls::InMemoryStore;
use ibc_client_tendermint::types::client_type as tm_client_type;
use ibc_core::host::types::identifiers::{ClientId, Sequence};
use ibc_core::host::ValidationContext;
use tokio::time::sleep;

use super::cosmos::helpers::dummy_signer;
use super::relayer::Relayer;
use crate::cosmos::builder::CosmosBuilder;
use crate::relayer::relay::MockRelayer;
use crate::sovereign::builder::DefaultBuilder;

/// Set ups a relayer between a mock sovereign chain and a mock cosmos chain
pub async fn sovereign_cosmos_setup(
    sov_builder: &mut DefaultBuilder,
    with_manual_tao: bool,
) -> Relayer {
    let mut sov_chain = sov_builder.build();

    let sov_client_counter = sov_chain.ibc_ctx().client_counter().unwrap();

    // TODO: this should be updated when there is a light client for sovereign chains
    let sov_client_id = ClientId::new(tm_client_type(), sov_client_counter).unwrap();

    let mut cos_builder = CosmosBuilder::default();

    let mut cos_chain = cos_builder.build_chain(InMemoryStore::default());

    let cos_client_counter = cos_chain.ibc_ctx().client_counter().unwrap();

    let cos_client_id = ClientId::new(tm_client_type(), cos_client_counter).unwrap();

    // Waits for the mock Cosmos chain to generate a few blocks before proceeding.
    sleep(Duration::from_secs(1)).await;

    if with_manual_tao {
        let sov_client_id = sov_chain.setup_client();

        let sov_conn_id = sov_chain.setup_connection(sov_client_id);

        let (sov_port_id, sov_chan_id) = sov_chain.setup_channel(sov_conn_id);

        sov_chain.with_send_sequence(sov_port_id, sov_chan_id, Sequence::from(1));

        let cos_client_id = cos_chain.setup_client();

        let cos_conn_id = cos_chain.setup_connection(cos_client_id);

        let (cos_port_id, cos_chan_id) = cos_chain.setup_channel(cos_conn_id);

        cos_chain.with_send_sequence(cos_port_id, cos_chan_id, Sequence::from(1));
    }

    MockRelayer::new(
        sov_chain.into(),
        cos_chain.into(),
        sov_client_id,
        cos_client_id,
        dummy_signer(),
        dummy_signer(),
    )
}
