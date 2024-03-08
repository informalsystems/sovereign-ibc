use alloc::vec::Vec;
use core::convert::TryFrom;

use ibc_core::client::context::client_state::ClientStateCommon;
use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use ibc_core::host::types::identifiers::ClientType;
use ibc_core::host::types::path::Path;
use ibc_core::primitives::proto::Any;
use sov_celestia_client_types::client_state::sov_client_type;

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
        _upgraded_client_state: Any,
        _upgraded_consensus_state: Any,
        _proof_upgrade_client: CommitmentProofBytes,
        _proof_upgrade_consensus_state: CommitmentProofBytes,
        _root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn verify_membership(
        &self,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _path: Path,
        _value: Vec<u8>,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn verify_non_membership(
        &self,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _path: Path,
    ) -> Result<(), ClientError> {
        Ok(())
    }
}
