use sov_bank::get_genesis_token_address;
use sov_modules_api::default_context::DefaultContext;

use crate::test_utils::builder::Builder;

#[test]
fn test_sdk_token_transfer() {
    let mut builder = Builder::new();

    let token = builder.get_tokens().first().unwrap();

    let token_address = get_genesis_token_address::<DefaultContext>(&token.token_name, token.salt);

    let sender_address = token.address_and_balances[0].0;

    let receiver_address = token.address_and_balances[1].0;

    let transfer_amount = 100;

    let expected_sender_balance = token.address_and_balances[0].1 - transfer_amount;

    let mut app = builder.build();

    app.setup_client();

    app.setup_connection();

    app.setup_channel();

    let msg_sdk_token_transfer = app.build_sdk_transfer(
        token_address,
        sender_address,
        receiver_address,
        transfer_amount,
    );

    app.send_ibc_message(msg_sdk_token_transfer);

    // Checks that the token has been transferred
    let escrowed_token = app
        .transfer()
        .escrowed_tokens
        .get(
            &token_address.to_string(),
            &mut app.working_set().borrow_mut(),
        )
        .unwrap();

    assert_eq!(escrowed_token, token_address);

    // Checks that the sender and receiver balances have been updated
    let sender_balance = app
        .bank()
        .get_balance_of(
            sender_address,
            token_address,
            &mut app.working_set().borrow_mut(),
        )
        .unwrap();
    assert_eq!(sender_balance, expected_sender_balance);
}

#[test]
fn test_recv_packet() {
    let mut builder = Builder::new();

    let mut app = builder.build();

    app.setup_client();

    app.setup_connection();

    app.setup_channel();

    let msg_recv_packet = app.build_recv_packet();

    app.send_ibc_message(msg_recv_packet);
}
