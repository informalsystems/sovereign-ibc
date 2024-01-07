use ibc_core::client::types::error::ClientError;
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::primitives::proto::Protobuf;

use crate::proto::RollupClientState as RawRollupClientState;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct RollupClientState {
    pub rollup_id: ChainId,
    pub post_root_state: Vec<u8>,
}

impl RollupClientState {
    pub fn new(rollup_id: ChainId, post_root_state: Vec<u8>) -> Self {
        Self {
            rollup_id,
            post_root_state,
        }
    }
}

impl Protobuf<RawRollupClientState> for RollupClientState {}

impl TryFrom<RawRollupClientState> for RollupClientState {
    type Error = ClientError;

    fn try_from(raw: RawRollupClientState) -> Result<Self, Self::Error> {
        let rollup_id = raw.rollup_id.parse().map_err(|_| ClientError::Other {
            description: "".into(),
        })?;

        Ok(Self::new(rollup_id, raw.post_root_state))
    }
}

impl From<RollupClientState> for RawRollupClientState {
    fn from(value: RollupClientState) -> Self {
        RawRollupClientState {
            rollup_id: value.rollup_id.to_string(),
            post_root_state: value.post_root_state,
        }
    }
}
