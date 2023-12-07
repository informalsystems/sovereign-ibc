use std::str::FromStr;
use std::time::Duration;

use ibc_app_transfer::types::{PrefixedDenom, TracePrefix};
use ibc_core::client::context::client_state::ClientStateCommon;
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::{Msg, Signer};
use ibc_testkit::fixtures::core::signer::{dummy_account_id, dummy_bech32_account};
use sov_bank::get_genesis_token_address;
use sov_ibc::clients::AnyClientState;
use sov_modules_api::default_context::DefaultContext;
use tokio::time::sleep;

use crate::relayer::handle::{Handle, QueryReq, QueryResp, QueryService};
use crate::setup::setup;

#[tokio::test]
async fn test_send_transfer_on_sov() {
    let (rly, mut rollup) = setup(true).await;

    // set transfer parameters
    let token = rollup.get_tokens().first().unwrap();
    let token_address = get_genesis_token_address::<DefaultContext>(&token.token_name, token.salt);
    let sender_on_sov = token.address_and_balances[0].0;
    let receiver_on_cos = dummy_account_id();
    let transfer_amount = 100;

    let expected_sender_balance = token.address_and_balances[0].1 - transfer_amount;

    let msg_sdk_token_transfer = rly.build_sdk_transfer_for_sov(
        token_address,
        Signer::from(sender_on_sov.to_string()),
        receiver_on_cos,
        transfer_amount,
    );

    rollup.apply_msg(vec![msg_sdk_token_transfer]).await;

    // Checks that the token has been transferred
    let escrowed_token = rly
        .src_chain_ctx()
        .service()
        .get_escrowed_token_address(token_address.to_string())
        .unwrap();

    assert_eq!(escrowed_token, token_address);

    // Checks that the sender balance have been updated
    let sender_balance = rly
        .src_chain_ctx()
        .service()
        .get_balance_of(sender_on_sov, token_address);

    assert_eq!(sender_balance, expected_sender_balance);
}

#[tokio::test]
async fn test_send_transfer_on_cos() {
    let (rly, mut rollup) = setup(true).await;

    // set transfer parameters
    let token = rollup.get_tokens().first().unwrap();
    let receiver_on_sov = token.address_and_balances[0].0;
    let sender_on_cos = dummy_bech32_account();
    let denom_on_cos = "basecoin";
    let transfer_amount = 100;

    let initial_sender_balance = rly
        .dst_chain_ctx()
        .service()
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

    let target_height = match rly.dst_chain_ctx().query(QueryReq::HostHeight) {
        QueryResp::HostHeight(height) => height,
        _ => panic!("unexpected response"),
    };

    let msg_update_client = rly.build_msg_update_client_for_sov(target_height);

    rollup.apply_msg(vec![msg_update_client]).await;

    let any_client_state = match rly
        .src_chain_ctx()
        .query(QueryReq::ClientState(rly.src_client_id().clone()))
    {
        QueryResp::ClientState(client_state) => client_state,
        _ => panic!("unexpected response"),
    };

    let client_state = AnyClientState::try_from(any_client_state).unwrap();

    assert_eq!(client_state.latest_height(), target_height);

    let msg_recv_packet = rly.build_msg_recv_packet_for_sov(target_height, msg_transfer_on_cos);

    rollup.apply_msg(vec![msg_recv_packet]).await;

    // Checks that the token has been transferred

    let denom_path_prefix = TracePrefix::new(PortId::transfer(), ChannelId::default());
    let mut prefixed_denom = PrefixedDenom::from_str(denom_on_cos).unwrap();
    prefixed_denom.add_trace_prefix(denom_path_prefix);

    let token_address_on_sov = rly
        .src_chain_ctx()
        .service()
        .get_minted_token_address(prefixed_denom.to_string())
        .unwrap();

    let receiver_balance = rly
        .src_chain_ctx()
        .service()
        .get_balance_of(receiver_on_sov, token_address_on_sov);

    assert_eq!(receiver_balance, transfer_amount);

    let sender_balance = rly
        .dst_chain_ctx()
        .service()
        .balance(denom_on_cos, sender_on_cos)
        .unwrap();

    assert_eq!(sender_balance, initial_sender_balance - transfer_amount);
}
