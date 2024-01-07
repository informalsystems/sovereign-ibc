// use sov_celestia_client_types::consensus_state::definition::SOVEREIGN_CONSENSUS_STATE_TYPE_URL;
// use sov_celestia_client_types::consensus_state::SovConsensusState;
// use sov_celestia_client_types::proto::ConsensusState as RawSovConsensusState;

// /// Newtype wrapper around the `ConsensusState` type imported from the
// /// `ibc-client-tendermint-types` crate. This wrapper exists so that we can
// /// bypass Rust's orphan rules and implement traits from
// /// `ibc::core::client::context` on the `ConsensusState` type.
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[derive(Clone, Debug, PartialEq)]
// pub struct ConsensusState(SovConsensusState);

// impl ConsensusState {
//     pub fn inner(&self) -> &SovConsensusState {
//         &self.0
//     }
// }

// impl From<SovConsensusState> for ConsensusState {
//     fn from(consensus_state: SovConsensusState) -> Self {
//         Self(consensus_state)
//     }
// }

// impl Protobuf<RawSovConsensusState> for ConsensusState {}

// impl TryFrom<RawSovConsensusState> for ConsensusState {
//     type Error = ClientError;

//     fn try_from(raw: RawSovConsensusState) -> Result<Self, Self::Error> {
//         Ok(Self(SovConsensusState::try_from(raw)?))
//     }
// }

// impl From<ConsensusState> for RawSovConsensusState {
//     fn from(client_state: ConsensusState) -> Self {
//         client_state.0.into()
//     }
// }

// impl Protobuf<Any> for ConsensusState {}

// impl TryFrom<Any> for ConsensusState {
//     type Error = ClientError;

//     fn try_from(raw: Any) -> Result<Self, Self::Error> {
//         Ok(Self(SovConsensusState::try_from(raw)?))
//     }
// }

// impl From<ConsensusState> for Any {
//     fn from(client_state: ConsensusState) -> Self {
//         client_state.0.into()
//     }
// }

// impl ConsensusStateTrait for ConsensusState {
//     fn root(&self) -> &CommitmentRoot {
//         &self.0.root
//     }

//     fn timestamp(&self) -> Timestamp {
//         let time = self.0.timestamp.unix_timestamp_nanos();
//         Timestamp::from_nanoseconds(time as u64).expect("invalid timestamp")
//     }

//     fn encode_vec(self) -> Vec<u8> {
//         <Self as Protobuf<Any>>::encode_vec(self)
//     }
// }
