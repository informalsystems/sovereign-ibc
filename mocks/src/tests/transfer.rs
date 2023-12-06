use std::str::FromStr;
use std::time::Duration;

use ibc_app_transfer::types::{PrefixedDenom, TracePrefix};
use ibc_core::client::context::client_state::ClientStateCommon;
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::host::ValidationContext;
use ibc_core::primitives::{Msg, Signer};
use ibc_testkit::fixtures::core::signer::{dummy_account_id, dummy_bech32_account};
use sov_bank::get_genesis_token_address;
use sov_modules_api::default_context::DefaultContext;
use tokio::time::sleep;

use crate::relayer::handle::Handle;
use crate::setup::sovereign_cosmos_setup;
use crate::sovereign::builder::DefaultBuilder;

#[tokio::test]
async fn test_send_transfer_on_sov() {
    let mut sov_builder = DefaultBuilder::default();

    // set transfer parameters
    let token = sov_builder.get_tokens().first().unwrap();
    let token_address = get_genesis_token_address::<DefaultContext>(&token.token_name, token.salt);
    let sender_on_sov = token.address_and_balances[0].0;
    let receiver_on_cos = dummy_account_id();
    let transfer_amount = 100;

    let expected_sender_balance = token.address_and_balances[0].1 - transfer_amount;

    let rly = sovereign_cosmos_setup(&mut sov_builder, true).await;

    let msg_sdk_token_transfer = rly.build_sdk_transfer_for_sov(
        token_address,
        Signer::from(sender_on_sov.to_string()),
        receiver_on_cos,
        transfer_amount,
    );

    rly.src_chain_ctx().send_msg(vec![msg_sdk_token_transfer]);

    // Checks that the token has been transferred
    let escrowed_token = rly
        .src_chain_ctx()
        .querier()
        .get_escrowed_token_address(token_address.to_string())
        .unwrap();

    assert_eq!(escrowed_token, token_address);

    // Checks that the sender balance have been updated
    let sender_balance = rly
        .src_chain_ctx()
        .querier()
        .get_balance_of(sender_on_sov, token_address);

    assert_eq!(sender_balance, expected_sender_balance);
}

#[tokio::test]
async fn test_send_transfer_on_cos() {
    let mut sov_builder = DefaultBuilder::default();

    // set transfer parameters
    let token = sov_builder.get_tokens().first().unwrap();
    let receiver_on_sov = token.address_and_balances[0].0;
    let sender_on_cos = dummy_bech32_account();
    let denom_on_cos = "basecoin";
    let transfer_amount = 100;

    let rly = sovereign_cosmos_setup(&mut sov_builder, true).await;

    let initial_sender_balance = rly
        .dst_chain_ctx()
        .querier()
        .balance(denom_on_cos, sender_on_cos.clone())
        .unwrap();

    let msg_transfer_on_cos = rly.build_msg_transfer_for_cos(
        denom_on_cos,
        Signer::from(sender_on_cos.clone()),
        Signer::from(receiver_on_sov.to_string()),
        transfer_amount,
    );

    rly.dst_chain_ctx()
        .send_msg(vec![msg_transfer_on_cos.clone().to_any()]);

    sleep(Duration::from_secs(1)).await;

    let target_height = rly.dst_chain_ctx().query_ibc().host_height().unwrap();

    let msg_update_client = rly.build_msg_update_client_for_sov(target_height);

    rly.src_chain_ctx().send_msg(vec![msg_update_client]);

    sleep(Duration::from_secs(1)).await;

    let cs = rly
        .src_chain_ctx()
        .query_ibc()
        .client_state(rly.src_client_id())
        .unwrap();

    assert_eq!(cs.latest_height(), target_height);

    let msg_recv_packet = rly.build_msg_recv_packet_for_sov(target_height, msg_transfer_on_cos);

    rly.src_chain_ctx().send_msg(vec![msg_recv_packet]);

    // Checks that the token has been transferred

    let denom_path_prefix = TracePrefix::new(PortId::transfer(), ChannelId::default());
    let mut prefixed_denom = PrefixedDenom::from_str(denom_on_cos).unwrap();
    prefixed_denom.add_trace_prefix(denom_path_prefix);

    let token_address_on_sov = rly
        .src_chain_ctx()
        .querier()
        .get_minted_token_address(prefixed_denom.to_string())
        .unwrap();

    let receiver_balance = rly
        .src_chain_ctx()
        .querier()
        .get_balance_of(receiver_on_sov, token_address_on_sov);

    assert_eq!(receiver_balance, transfer_amount);

    let sender_balance = rly
        .dst_chain_ctx()
        .querier()
        .balance(denom_on_cos, sender_on_cos)
        .unwrap();

    assert_eq!(sender_balance, initial_sender_balance - transfer_amount);
}
