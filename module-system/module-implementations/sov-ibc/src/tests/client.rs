use std::time::Duration;

use ibc::core::ics02_client::client_state::ClientStateCommon;
use ibc::core::ValidationContext;
use tokio::time::sleep;

use crate::test_utils::relayer::handle::Handle;
use crate::test_utils::setup::sovereign_cosmos_setup;
use crate::test_utils::sovereign::builder::Builder;

#[tokio::test]
async fn test_create_client() {
    let mut sovereign_builder = Builder::default();

    let rly = sovereign_cosmos_setup(&mut sovereign_builder, false).await;

    let msg_create_client = rly.build_msg_create_client();

    rly.src_chain_ctx().send_msg(vec![msg_create_client]);

    let client_counter = rly.src_chain_ctx().query_ibc().client_counter().unwrap();

    assert_eq!(client_counter, 1);

    assert!(rly
        .src_chain_ctx()
        .query_ibc()
        .client_state(rly.src_client_id())
        .is_ok());
}

#[tokio::test]
async fn test_update_client() {
    let mut sovereign_builder = Builder::default();

    let rly = sovereign_cosmos_setup(&mut sovereign_builder, false).await;

    let msg_create_client = rly.build_msg_create_client();

    rly.src_chain_ctx().send_msg(vec![msg_create_client]);

    // Waits for the mock cosmos chain to progress a few blocks
    sleep(Duration::from_secs(1)).await;

    let target_height = rly.dst_chain_ctx().query_ibc().host_height().unwrap();

    let msg_update_client = rly.build_msg_update_client_for_sov(target_height);

    rly.src_chain_ctx().send_msg(vec![msg_update_client]);

    let client_state = rly
        .src_chain_ctx()
        .query_ibc()
        .client_state(rly.src_client_id())
        .unwrap();

    assert_eq!(client_state.latest_height(), target_height);
}
