use std::str::FromStr;

use ibc_core::commitment_types::commitment::CommitmentRoot;
use sov_celestia_client::consensus_state::ConsensusState as HostConsensusState;
use sov_celestia_client::types::consensus_state::{
    ConsensusState as SovConsensusState, TmConsensusParams,
};
pub use sov_mock_da::{MockAddress, MockDaConfig, MockDaService, MockDaSpec};
use sov_rollup_interface::da::BlockHeaderTrait;

use crate::HasConsensusState;

impl HasConsensusState for MockDaSpec {
    fn consensus_state(header: &Self::BlockHeader, user_hash: [u8; 32]) -> HostConsensusState {
        let sovereign_params = CommitmentRoot::from_bytes(&user_hash).into();

        let da_params = TmConsensusParams::new(
            tendermint::Time::from_unix_timestamp(
                header.time().secs(),
                header.time().subsec_nanos(),
            )
            .expect("time is valid"),
            tendermint::Hash::from_str(
                "D6B93922C33AAEBEC9043566CB4B1B48365B1358B67C7DEF986D9EE1861BC143",
            )
            .expect("Never fails"),
        );

        SovConsensusState {
            sovereign_params,
            da_params,
        }
        .into()
    }
}
