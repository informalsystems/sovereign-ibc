use ibc::clients::ics07_tendermint::client_type as tm_client_type;
use ibc::core::ics24_host::identifier::ClientId;
use ibc::core::ValidationContext;

use crate::test_utils::builder::Builder;

#[test]
fn test_create_client() {
    let mut builder = Builder::default();

    let app = builder.build();

    let msg_create_client = app.build_msg_create_client();

    app.send_ibc_message(msg_create_client);

    let client_counter = app.ibc_ctx().client_counter().unwrap();

    assert_eq!(client_counter, 1);

    let client_id = ClientId::new(tm_client_type(), 0).unwrap();

    assert!(app.ibc_ctx().client_state(&client_id).is_ok());
}

#[test]
fn test_update_client() {
    let mut builder = Builder::default();

    let mut app = builder.build();

    app.setup_client();

    let msg_update_client = app.build_msg_update_client();

    app.send_ibc_message(msg_update_client);
}
