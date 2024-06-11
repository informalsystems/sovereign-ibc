use ibc_core::commitment_types::commitment::CommitmentRoot;
use ibc_core::primitives::proto::Protobuf;
pub use sov_celestia_adapter::verifier::CelestiaSpec;
pub use sov_celestia_adapter::*;
use sov_celestia_client::consensus_state::ConsensusState as HostConsensusState;
use sov_celestia_client::types::consensus_state::{
    ConsensusState as SovConsensusState, TmConsensusParams,
};

use crate::HasConsensusState;

impl HasConsensusState for CelestiaSpec {
    fn consensus_state(header: &CelestiaHeader, user_hash: [u8; 32]) -> HostConsensusState {
        let sovereign_params = CommitmentRoot::from_bytes(&user_hash).into();

        let protobuf_time = tendermint::time::Time::decode(header.header.time.as_slice())
            .expect("Timestamp must be valid");

        let da_params = TmConsensusParams::new(
            tendermint::Time::from_unix_timestamp(
                protobuf_time.unix_timestamp(),
                u32::try_from(protobuf_time.unix_timestamp_nanos() / 1_000_000_000)
                    .expect("no overflow"),
            )
            .expect("Could not obtain timestamp from header"),
            tendermint::Hash::decode_vec(&header.header.next_validators_hash)
                .expect("Could not decode next validator hash from header"),
        );

        SovConsensusState {
            sovereign_params,
            da_params,
        }
        .into()
    }
}
