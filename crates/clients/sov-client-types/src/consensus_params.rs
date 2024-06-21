use ibc_core::client::types::error::ClientError;
use ibc_core::commitment_types::commitment::CommitmentRoot;
use ibc_core::primitives::proto::Protobuf;

use crate::proto::SovereignConsensusParams as RawSovereignConsensusParams;

/// Defines the Sovereign SDK rollup-specific consensus parameters.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SovereignConsensusParams {
    /// The commitment root of the rollup's final state.
    ///
    /// This is obtained on client updates from the `final_state_root` field of
    /// the received `AggregatedProof`.
    pub root: CommitmentRoot,
}

impl SovereignConsensusParams {
    pub fn new(root: CommitmentRoot) -> Self {
        Self { root }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.root.as_bytes()
    }
}

impl From<CommitmentRoot> for SovereignConsensusParams {
    fn from(root: CommitmentRoot) -> Self {
        Self { root }
    }
}

impl From<&[u8]> for SovereignConsensusParams {
    fn from(value: &[u8]) -> Self {
        Self {
            root: CommitmentRoot::from_bytes(value),
        }
    }
}

impl Protobuf<RawSovereignConsensusParams> for SovereignConsensusParams {}

impl TryFrom<RawSovereignConsensusParams> for SovereignConsensusParams {
    type Error = ClientError;

    fn try_from(raw: RawSovereignConsensusParams) -> Result<Self, Self::Error> {
        let root = raw
            .root
            .ok_or(ClientError::Other {
                description: "missing root".to_string(),
            })?
            .hash;

        Ok(Self { root: root.into() })
    }
}

impl From<SovereignConsensusParams> for RawSovereignConsensusParams {
    fn from(value: SovereignConsensusParams) -> Self {
        Self {
            root: Some(ibc_core::commitment_types::proto::v1::MerkleRoot {
                hash: value.root.into_vec(),
            }),
        }
    }
}
