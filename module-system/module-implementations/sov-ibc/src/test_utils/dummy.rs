use std::time::Duration;

use ibc::clients::ics07_tendermint::client_state::{AllowUpdate, ClientState};
use ibc::core::ics24_host::identifier::ChainId;
use ibc::Height;

pub fn dummy_tm_client_state(chain_id: ChainId, latest_hight: Height) -> ClientState {
    ClientState::new(
        chain_id,
        Default::default(),
        Duration::from_secs(64000),
        Duration::from_secs(128000),
        Duration::from_millis(3000),
        latest_hight,
        Default::default(),
        Default::default(),
        AllowUpdate {
            after_expiry: false,
            after_misbehaviour: false,
        },
    )
    .unwrap()
}
