use std::str::FromStr;

use ibc_app_transfer::types::{PrefixedDenom, TracePrefix};
use ibc_core::client::context::client_state::ClientStateCommon;
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::ToProto;
use sov_bank::{get_genesis_token_address, TokenConfig};
use sov_ibc::call::CallMessage;
use sov_ibc::clients::AnyClientState;
use sov_modules_api::default_context::DefaultContext;
use test_log::test;

use crate::configs::TransferTestConfig;
use crate::relayer::{Handle, QueryReq, QueryResp, QueryService, RelayerBuilder};

/// Checks if a transfer initiated on the rollup (`send_transfer`) succeeds by
/// escrowing the token on the rollup and creating a new token on the Cosmos
/// chain (`recv_packet`).
#[test(tokio::test)]
async fn test_escrow_unescrow_on_sov() {
    let relayer_builder = RelayerBuilder::default();

    let rly = relayer_builder.clone().with_manual_tao().setup().await;

    // set transfer parameters
    let token: TokenConfig<DefaultContext> = relayer_builder.setup_cfg().get_tokens()[0].clone();
    let token_address = get_genesis_token_address::<DefaultContext>(&token.token_name, token.salt);
    let mut cfg = TransferTestConfig::builder()
        .sov_denom(token.token_name.clone())
        .sov_token_address(Some(token_address))
        .sov_address(token.address_and_balances[0].0)
        .build();

    let expected_sender_balance = token.address_and_balances[0].1 - cfg.amount * 2;

    // -----------------------------------------------------------------------
    // Send a `MsgTransfer` to the rollup
    // -----------------------------------------------------------------------
    let msg_transfer_on_sov = rly.build_msg_transfer_for_sov(&cfg).await;

    rly.src_chain_ctx()
        .submit_msgs(vec![
            CallMessage::Transfer(msg_transfer_on_sov.clone()).into()
        ])
        .await;

    // -----------------------------------------------------------------------
    // Check that the token has been escrowed
    // -----------------------------------------------------------------------
    let escrowed_token = rly
        .src_chain_ctx()
        .service()
        .get_escrowed_token_address(token_address.to_string())
        .unwrap();

    assert_eq!(escrowed_token, token_address);

    // -----------------------------------------------------------------------
    // Transfer the same token once again
    // -----------------------------------------------------------------------
    rly.src_chain_ctx()
        .submit_msgs(vec![
            CallMessage::Transfer(msg_transfer_on_sov.clone()).into()
        ])
        .await;

    // -----------------------------------------------------------------------
    // Check the sender balance has been updated correctly
    // -----------------------------------------------------------------------
    let sender_balance = rly
        .src_chain_ctx()
        .service()
        .get_balance_of(cfg.sov_address, token_address);

    assert_eq!(sender_balance, expected_sender_balance);

    // -----------------------------------------------------------------------
    // Transfer another token but with the same name as the previous one
    // -----------------------------------------------------------------------

    let fake_token_message = rly.build_msg_create_token(&token);

    rly.src_chain_ctx()
        .submit_msgs(vec![fake_token_message.clone().into()])
        .await;

    let fake_token_address = relayer_builder.setup_cfg().get_token_address(&token);

    cfg.sov_token_address = Some(fake_token_address);
    cfg.amount = 50;

    let fake_token_sender_initial_balance = rly
        .src_chain_ctx()
        .service()
        .get_balance_of(cfg.sov_address, fake_token_address);

    let msg_transfer_on_sov = rly.build_msg_transfer_for_sov(&cfg).await;

    rly.src_chain_ctx()
        .submit_msgs(vec![
            CallMessage::Transfer(msg_transfer_on_sov.clone()).into()
        ])
        .await;

    // -----------------------------------------------------------------------
    // Check that the token has been escrowed as a distinct asset
    // -----------------------------------------------------------------------
    let escrowed_token = rly
        .src_chain_ctx()
        .service()
        .get_escrowed_token_address(fake_token_address.to_string())
        .unwrap();

    assert_eq!(escrowed_token, fake_token_address);

    let sender_genuine_token_balance = rly
        .src_chain_ctx()
        .service()
        .get_balance_of(cfg.sov_address, token_address);

    assert_eq!(sender_genuine_token_balance, expected_sender_balance);

    let fake_token_sender_balance = rly
        .src_chain_ctx()
        .service()
        .get_balance_of(cfg.sov_address, fake_token_address);

    assert_eq!(
        fake_token_sender_balance,
        fake_token_sender_initial_balance - cfg.amount
    );
}

/// Checks if a transfer initiated on the Cosmos chain (`send_transfer`)
/// succeeds by creating a new token on the rollup (`recv_packet`).
#[test(tokio::test)]
#[ignore]
async fn test_mint_burn_on_sov() {
    let relayer_builder = RelayerBuilder::default();

    let rly = relayer_builder.clone().with_manual_tao().setup().await;

    // set transfer parameters
    let token = relayer_builder.setup_cfg().get_tokens()[0].clone();
    let mut cfg = TransferTestConfig::builder()
        .sov_denom(token.token_name.clone())
        .sov_address(token.address_and_balances[0].0)
        .build();

    let fake_token = TokenConfig {
        token_name: "transfer/channel-0/basecoin".to_string(),
        ..token.clone()
    };

    let fake_token_message = rly.build_msg_create_token(&fake_token);

    rly.src_chain_ctx()
        .submit_msgs(vec![fake_token_message.clone().into()])
        .await;

    let fake_minted_token_address = relayer_builder.setup_cfg().get_token_address(&token);

    // Store the current balance of the sender to check it later after the transfers
    let initial_sender_balance = rly
        .dst_chain_ctx()
        .service()
        .get_balance_of(&cfg.cos_denom, cfg.cos_address.clone())
        .unwrap();

    // -----------------------------------------------------------------------
    // Send a `MsgTransfer` to the Cosmos chain
    // -----------------------------------------------------------------------
    let msg_transfer_on_cos = rly.build_msg_transfer_for_cos(&cfg).await;

    rly.dst_chain_ctx()
        .submit_msgs(vec![msg_transfer_on_cos.clone().to_any()])
        .await;

    // -----------------------------------------------------------------------
    // Send a `MsgRecvPacket` paired with a `MsgUpdateClient` to the rollup
    // -----------------------------------------------------------------------
    let target_height = match rly.dst_chain_ctx().query(QueryReq::HostHeight).await {
        QueryResp::HostHeight(height) => height,
        _ => panic!("unexpected response"),
    };

    let msg_update_client = rly.build_msg_update_client_for_sov(target_height).await;

    let msg_recv_packet = rly
        .build_msg_recv_packet_for_sov(target_height, msg_transfer_on_cos.clone())
        .await;

    rly.src_chain_ctx()
        .submit_msgs(vec![msg_update_client.into(), msg_recv_packet.into()])
        .await;

    // -----------------------------------------------------------------------
    // Check client state has been updated successfully
    // -----------------------------------------------------------------------
    let any_client_state = match rly
        .src_chain_ctx()
        .query(QueryReq::ClientState(rly.dst_client_id().clone()))
        .await
    {
        QueryResp::ClientState(client_state) => client_state,
        _ => panic!("unexpected response"),
    };

    let client_state = AnyClientState::try_from(any_client_state).unwrap();

    assert_eq!(client_state.latest_height(), target_height);

    // -----------------------------------------------------------------------
    // Check uniqueness of the created token address
    // -----------------------------------------------------------------------
    let denom_path_prefix = TracePrefix::new(PortId::transfer(), ChannelId::default());
    let mut prefixed_denom = PrefixedDenom::from_str(&cfg.cos_denom).unwrap();
    prefixed_denom.add_trace_prefix(denom_path_prefix);

    let token_address_on_sov = rly
        .src_chain_ctx()
        .service()
        .get_minted_token_address(prefixed_denom.to_string())
        .unwrap();

    assert_ne!(token_address_on_sov, fake_minted_token_address);

    // -----------------------------------------------------------------------
    // Transfer the same token once again
    // -----------------------------------------------------------------------
    rly.dst_chain_ctx()
        .submit_msgs(vec![msg_transfer_on_cos.clone().to_any()])
        .await;

    let target_height = match rly.dst_chain_ctx().query(QueryReq::HostHeight).await {
        QueryResp::HostHeight(height) => height,
        _ => panic!("unexpected response"),
    };

    let msg_update_client = rly.build_msg_update_client_for_sov(target_height).await;

    let msg_recv_packet = rly
        .build_msg_recv_packet_for_sov(target_height, msg_transfer_on_cos.clone())
        .await;

    rly.src_chain_ctx()
        .submit_msgs(vec![msg_update_client.into(), msg_recv_packet.into()])
        .await;

    // -----------------------------------------------------------------------
    // Check the token has been minted on the rollup and escrowed on the Cosmos chain
    // -----------------------------------------------------------------------
    let receiver_balance = rly
        .src_chain_ctx()
        .service()
        .get_balance_of(cfg.sov_address, token_address_on_sov);

    let mut expected_receiver_balance = cfg.amount * 2;

    assert_eq!(receiver_balance, expected_receiver_balance);

    let sender_balance = rly
        .dst_chain_ctx()
        .service()
        .get_balance_of(&cfg.cos_denom, cfg.cos_address.clone())
        .unwrap();

    let expected_sender_balance = initial_sender_balance - cfg.amount * 2;

    assert_eq!(sender_balance, expected_sender_balance);

    // -----------------------------------------------------------------------
    // Send back the token to the Cosmos chain
    // -----------------------------------------------------------------------

    cfg.sov_denom = "transfer/channel-0/basecoin".to_string();
    cfg.sov_token_address = Some(token_address_on_sov);

    let msg_transfer_on_sov = rly.build_msg_transfer_for_sov(&cfg).await;

    rly.src_chain_ctx()
        .submit_msgs(vec![
            CallMessage::Transfer(msg_transfer_on_sov.clone()).into()
        ])
        .await;

    // TODO: Uncomment this part when the rollup header can be queried by the relayer
    //
    // let target_height = match rly.src_chain_ctx().query(QueryReq::HostHeight).await {
    //     QueryResp::HostHeight(height) => height,
    //     _ => panic!("unexpected response"),
    // };

    // let msg_update_client = rly.build_msg_update_client_for_cos(target_height).await;

    // let msg_recv_packet = rly
    //     .build_msg_recv_packet_for_cos(target_height, msg_transfer_on_sov)
    //     .await;

    // rly.dst_chain_ctx()
    //     .submit_msgs(vec![msg_update_client, msg_recv_packet.to_any()])
    //     .await;

    // -----------------------------------------------------------------------
    // Check the token has been burned on rollup and unescrowed on Cosmos chain
    // -----------------------------------------------------------------------
    let sender_balance = rly
        .src_chain_ctx()
        .service()
        .get_balance_of(cfg.sov_address, token_address_on_sov);

    expected_receiver_balance -= cfg.amount;

    assert_eq!(sender_balance, expected_receiver_balance);

    // TODO: Uncomment this part when the rollup header can be queried by the relayer

    // let receiver_balance = rly
    //     .dst_chain_ctx()
    //     .service()
    //     .get_balance_of(denom_on_cos, sender_on_cos)
    //     .unwrap();

    // expected_sender_balance += transfer_amount;

    // assert_eq!(receiver_balance, expected_sender_balance);
}
