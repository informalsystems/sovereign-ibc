use alloc::vec::Vec;

use borsh::de::BorshDeserialize;
use borsh::BorshSerialize;
use ibc_core::client::context::client_state::ClientStateCommon;
use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core::client::types::error::{ClientError, UpgradeClientError};
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use ibc_core::commitment_types::error::CommitmentError;
use ibc_core::host::types::identifiers::ClientType;
use ibc_core::host::types::path::{Path, UpgradeClientPath};
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::proto::Any;
use ibc_core::primitives::ToVec;
use jmt::proof::SparseMerkleProof;
use sov_celestia_client_types::client_state::{sov_client_type, SovTmClientState};

use super::ClientState;
use crate::consensus_state::ConsensusState;

impl ClientStateCommon for ClientState {
    fn verify_consensus_state(&self, consensus_state: Any) -> Result<(), ClientError> {
        let tm_consensus_state = ConsensusState::try_from(consensus_state)?;
        if tm_consensus_state.root().is_empty() {
            return Err(ClientError::Other {
                description: "empty commitment root".into(),
            });
        };

        Ok(())
    }

    fn client_type(&self) -> ClientType {
        sov_client_type()
    }

    fn latest_height(&self) -> Height {
        self.0.latest_height()
    }

    fn validate_proof_height(&self, proof_height: Height) -> Result<(), ClientError> {
        if self.latest_height() < proof_height {
            return Err(ClientError::InvalidProofHeight {
                latest_height: self.latest_height(),
                proof_height,
            });
        }
        Ok(())
    }

    /// Perform client-specific verifications and check all data in the new
    /// client state to be the same across all valid clients for the new chain.
    ///
    /// You can learn more about how to upgrade IBC-connected SDK chains in
    /// [this](https://ibc.cosmos.network/main/ibc/upgrades/quick-guide.html)
    /// guide
    fn verify_upgrade_client(
        &self,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
        proof_upgrade_client: CommitmentProofBytes,
        proof_upgrade_consensus_state: CommitmentProofBytes,
        root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        verify_upgrade_client(
            self.inner(),
            upgraded_client_state,
            upgraded_consensus_state,
            proof_upgrade_client,
            proof_upgrade_consensus_state,
            root,
        )
    }

    fn verify_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
        value: Vec<u8>,
    ) -> Result<(), ClientError> {
        verify_membership(prefix, proof, root, path, value)
    }

    fn verify_non_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
    ) -> Result<(), ClientError> {
        verify_non_membership(prefix, proof, root, path)
    }
}

/// Perform client-specific verifications and check all data in the new client
/// state to be the same across all valid Sovereign clients for the new rollup.
pub fn verify_upgrade_client(
    client_state: &SovTmClientState,
    upgraded_client_state: Any,
    upgraded_consensus_state: Any,
    proof_upgrade_client: CommitmentProofBytes,
    proof_upgrade_consensus_state: CommitmentProofBytes,
    root: &CommitmentRoot,
) -> Result<(), ClientError> {
    // Make sure that the client type is of Tendermint type `ClientState`
    let upgraded_tm_client_state = ClientState::try_from(upgraded_client_state.clone())?;

    // Make sure that the consensus type is of Tendermint type `ConsensusState`
    ConsensusState::try_from(upgraded_consensus_state.clone())?;

    let latest_height = client_state.latest_height;

    let upgraded_tm_client_state_height = upgraded_tm_client_state.latest_height();

    // Make sure the latest height of the current client is not greater than the
    // upgrade height This condition checks both the revision number and the
    // height
    if latest_height >= upgraded_tm_client_state_height {
        Err(UpgradeClientError::LowUpgradeHeight {
            upgraded_height: latest_height,
            client_height: upgraded_tm_client_state_height,
        })?;
    }

    let upgrade_path_prefix = client_state.upgrade_path.clone().try_into()?;

    let last_height = latest_height.revision_height();

    // Verify the proof of the upgraded client state
    verify_membership(
        &upgrade_path_prefix,
        &proof_upgrade_client,
        root,
        Path::UpgradeClient(UpgradeClientPath::UpgradedClientState(last_height)),
        upgraded_client_state.to_vec(),
    )?;

    // Verify the proof of the upgraded consensus state
    verify_membership(
        &upgrade_path_prefix,
        &proof_upgrade_consensus_state,
        root,
        Path::UpgradeClient(UpgradeClientPath::UpgradedClientConsensusState(last_height)),
        upgraded_consensus_state.to_vec(),
    )?;

    Ok(())
}

pub fn verify_membership(
    prefix: &CommitmentPrefix,
    proof: &CommitmentProofBytes,
    root: &CommitmentRoot,
    path: Path,
    value: Vec<u8>,
) -> Result<(), ClientError> {
    let root_bytes: [u8; 32] = root.as_bytes().try_into().map_err(|_| ClientError::Other {
        description: "invalid commitment root, expected 32 bytes".into(),
    })?;

    let key_hash = obtain_key_hash(prefix, path)?;

    let proof = SparseMerkleProof::<sha2::Sha256>::try_from_slice(proof.as_ref()).map_err(|e| {
        ClientError::InvalidCommitmentProof(CommitmentError::DecodingFailure(e.to_string()))
    })?;

    proof
        .verify_existence(root_bytes.into(), key_hash, value)
        .map_err(|e| ClientError::Other {
            description: e.to_string(),
        })?;

    Ok(())
}

pub fn verify_non_membership(
    prefix: &CommitmentPrefix,
    proof: &CommitmentProofBytes,
    root: &CommitmentRoot,
    path: Path,
) -> Result<(), ClientError> {
    let root_bytes: [u8; 32] = root.as_bytes().try_into().map_err(|_| ClientError::Other {
        description: "invalid commitment root".into(),
    })?;

    let key_hash = obtain_key_hash(prefix, path)?;

    let proof = SparseMerkleProof::<sha2::Sha256>::try_from_slice(proof.as_ref()).map_err(|e| {
        ClientError::InvalidCommitmentProof(CommitmentError::DecodingFailure(e.to_string()))
    })?;

    proof
        .verify_nonexistence(root_bytes.into(), key_hash)
        .map_err(|_| ClientError::InvalidCommitmentProof(CommitmentError::VerificationFailure))?;

    Ok(())
}

/// Obtain the JMT key hash for the given path and prefix.
fn obtain_key_hash(prefix: &CommitmentPrefix, path: Path) -> Result<jmt::KeyHash, ClientError> {
    let (prefix_map, encoded_key) = match path {
        Path::ClientState(p) => ("client_state_map", p.try_to_vec()),
        Path::ClientConsensusState(p) => ("consensus_state_map", p.try_to_vec()),
        Path::Connection(p) => ("connection_end_map", p.try_to_vec()),
        Path::ChannelEnd(p) => ("channel_end_map", p.try_to_vec()),
        Path::SeqSend(p) => ("send_sequence_map", p.try_to_vec()),
        Path::SeqRecv(p) => ("recv_sequence_map", p.try_to_vec()),
        Path::SeqAck(p) => ("ack_sequence_map", p.try_to_vec()),
        Path::Commitment(p) => ("packet_commitment_map", p.try_to_vec()),
        Path::Ack(p) => ("packet_ack_map", p.try_to_vec()),
        Path::Receipt(p) => ("packet_receipt_map", p.try_to_vec()),
        Path::UpgradeClient(p) => match p {
            UpgradeClientPath::UpgradedClientState(_) => {
                ("upgraded_client_state_map", p.try_to_vec())
            }
            UpgradeClientPath::UpgradedClientConsensusState(_) => {
                ("upgraded_consensus_state_map", p.try_to_vec())
            }
        },
        _ => Err(ClientError::Other {
            description: "unsupported path".into(),
        })?,
    };

    let encoded_key = encoded_key.map_err(|_| ClientError::Other {
        description: "failed to encode key".into(),
    })?;

    let key_bytes = compute_key_bytes(prefix, prefix_map, encoded_key);

    Ok(jmt::KeyHash::with::<sha2::Sha256>(key_bytes.as_slice()))
}

/// Compute the key bytes for the given path and prefix.
fn compute_key_bytes(prefix: &CommitmentPrefix, prefix_map: &str, encoded_key: Vec<u8>) -> Vec<u8> {
    let prefix_bytes = prefix.as_bytes();
    let state_map_bytes = prefix_map.as_bytes();

    let mut key_bytes =
        Vec::with_capacity(prefix_bytes.len() + state_map_bytes.len() + 1 + encoded_key.len());

    key_bytes.extend_from_slice(prefix_bytes);
    key_bytes.extend_from_slice(state_map_bytes);
    key_bytes.push(b'/');
    key_bytes.extend_from_slice(&encoded_key);

    key_bytes
}
