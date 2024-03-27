use ibc_core::commitment_types::commitment::CommitmentRoot;
use ibc_core::primitives::proto::Protobuf;
pub use sov_celestia_adapter::verifier::CelestiaSpec;
pub use sov_celestia_adapter::*;
use sov_celestia_client::consensus_state::ConsensusState as HostConsensusState;
use sov_celestia_client::types::consensus_state::{
    ConsensusState as SovConsensusState, TmConsensusParams,
};
use sov_rollup_interface::da::BlockHeaderTrait;

use crate::HasConsensusState;

impl HasConsensusState for CelestiaSpec {
    fn consensus_state(header: &CelestiaHeader) -> HostConsensusState {
        let timestamp = tendermint::Time::from_unix_timestamp(
            header.time().secs(),
            header.time().subsec_nanos(),
        )
        .expect("Could not obtain timestamp from header");

        let next_validator_hash = tendermint::Hash::decode_vec(&header.header.next_validators_hash)
            .expect("Could not decode next validator hash from header");

        SovConsensusState {
            root: CommitmentRoot::from_bytes(header.hash().as_ref()),
            da_params: TmConsensusParams::new(timestamp, next_validator_hash),
        }
        .into()
    }
}
