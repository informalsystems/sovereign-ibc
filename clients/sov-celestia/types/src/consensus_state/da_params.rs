use ibc_core::primitives::proto::Protobuf;
use ibc_proto_sov::ibc::lightclients::sovereign::tendermint::v1::TendermintConsensusParams as RawTmConsensusParams;
use tendermint::hash::Algorithm;
use tendermint::{Hash, Time};
use tendermint_proto::google::protobuf as tpb;

use crate::error::Error;

/// Defines the Tendermint-specific consensus state parameters
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TmConsensusParams {
    pub timestamp: Time,
    pub next_validators_hash: Hash,
}

impl TmConsensusParams {
    pub fn new(timestamp: Time, next_validators_hash: Hash) -> Self {
        Self {
            timestamp,
            next_validators_hash,
        }
    }
}

impl Protobuf<RawTmConsensusParams> for TmConsensusParams {}

impl TryFrom<RawTmConsensusParams> for TmConsensusParams {
    type Error = Error;

    fn try_from(raw: RawTmConsensusParams) -> Result<Self, Self::Error> {
        let ibc_proto_sov::google::protobuf::Timestamp { seconds, nanos } =
            raw.timestamp.ok_or(Error::missing("timestamp"))?;

        let proto_timestamp = tpb::Timestamp { seconds, nanos };
        let timestamp = proto_timestamp
            .try_into()
            .map_err(|_| Error::invalid("invalid timestamp"))?;

        let next_validators_hash = Hash::from_bytes(Algorithm::Sha256, &raw.next_validators_hash)
            .map_err(|_| Error::invalid("invalid next validators hash"))?;

        Ok(Self::new(timestamp, next_validators_hash))
    }
}

impl From<TmConsensusParams> for RawTmConsensusParams {
    fn from(value: TmConsensusParams) -> Self {
        let tpb::Timestamp { seconds, nanos } = value.timestamp.into();
        let timestamp = ibc_proto_sov::google::protobuf::Timestamp { seconds, nanos };

        Self {
            timestamp: Some(timestamp),
            next_validators_hash: value.next_validators_hash.as_bytes().to_vec(),
        }
    }
}
