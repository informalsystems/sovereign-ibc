use std::time::Duration;

use ibc::core::ics02_client::client_state::ClientStateCommon;
use ibc::core::ValidationContext;
use sov_bank::get_genesis_token_address;
use sov_modules_api::default_context::DefaultContext;
use tokio::time::sleep;

use crate::test_utils::relayer::handle::Handle;
use crate::test_utils::relayer::relay::build_msg_recv_packet_for_sov;
use crate::test_utils::setup::sovereign_cosmos_setup;
use crate::test_utils::sovereign::builder::Builder;

#[tokio::test]
async fn test_sdk_token_transfer() {
    let mut src_builder = Builder::default();

    let token = src_builder.get_tokens().first().unwrap();

    let token_address = get_genesis_token_address::<DefaultContext>(&token.token_name, token.salt);

    let sender_address = token.address_and_balances[0].0;

    let receiver_address = token.address_and_balances[1].0;

    let transfer_amount = 100;

    let expected_sender_balance = token.address_and_balances[0].1 - transfer_amount;

    let rly = sovereign_cosmos_setup(&mut src_builder, true).await;

    let msg_sdk_token_transfer = rly.src_chain_ctx().build_sdk_transfer(
        token_address,
        sender_address,
        receiver_address,
        transfer_amount,
    );

    rly.src_chain_ctx().send_msg(vec![msg_sdk_token_transfer]);

    // Checks that the token has been transferred
    let escrowed_token = rly
        .src_chain_ctx()
        .querier()
        .get_escrow_address(token_address)
        .unwrap();

    assert_eq!(escrowed_token, token_address);

    // Checks that the sender balance have been updated
    let sender_balance = rly
        .src_chain_ctx()
        .querier()
        .get_balance_of(sender_address, token_address);

    assert_eq!(sender_balance, expected_sender_balance);
}

// FIXME: This test already fails as there must be a send packet on the mock
// cosmos chain
#[tokio::test]
async fn test_recv_packet() {
    let mut src_builder = Builder::default();

    let rly = sovereign_cosmos_setup(&mut src_builder, true).await;

    let msg_create_client = rly.build_msg_create_client();

    rly.src_chain_ctx().send_msg(vec![msg_create_client]);

    sleep(Duration::from_secs(1)).await;

    let target_height = rly.dst_chain_ctx().query_ibc().host_height().unwrap();

    let msg_update_client = rly.build_msg_update_client_for_sov(target_height);

    rly.src_chain_ctx().send_msg(vec![msg_update_client]);

    let cs = rly
        .src_chain_ctx()
        .query_ibc()
        .client_state(rly.src_client_id())
        .unwrap();

    let msg_recv_packet = build_msg_recv_packet_for_sov(&rly, cs.latest_height()).await;

    rly.src_chain_ctx().send_msg(vec![msg_recv_packet]);
}
