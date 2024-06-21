use ibc_client_cw::api::ClientType;
use sov_celestia_client::client_state::ClientState;
use sov_celestia_client::consensus_state::ConsensusState;

pub struct SovTmClient;

impl<'a> ClientType<'a> for SovTmClient {
    type ClientState = ClientState;
    type ConsensusState = ConsensusState;
}
