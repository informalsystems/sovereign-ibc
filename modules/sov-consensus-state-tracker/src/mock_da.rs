use ibc_core::commitment_types::commitment::CommitmentRoot;
use sov_celestia_client::consensus_state::ConsensusState as HostConsensusState;
use sov_celestia_client::types::consensus_state::{
    ConsensusState as SovConsensusState, TmConsensusParams,
};
pub use sov_mock_da::{MockAddress, MockDaConfig, MockDaService, MockDaSpec};
use sov_rollup_interface::da::BlockHeaderTrait;

use crate::HasConsensusState;

impl HasConsensusState for MockDaSpec {
    fn consensus_state(header: &Self::BlockHeader) -> HostConsensusState {
        SovConsensusState {
            root: CommitmentRoot::from_bytes(header.hash().as_ref()),
            da_params: TmConsensusParams::new(
                tendermint::Time::from_unix_timestamp(
                    header.time().secs(),
                    header.time().subsec_nanos(),
                )
                .expect("time is valid"),
                tendermint::Hash::None,
            ),
        }
        .into()
    }
}
