#[cfg(feature = "celestia-da")]
mod celestia_da {
    use const_rollup_config::{ROLLUP_BATCH_NAMESPACE_RAW, ROLLUP_PROOF_NAMESPACE_RAW};
    use sov_consensus_state_tracker::types::Namespace;
    use sov_consensus_state_tracker::verifier::RollupParams;
    use sov_consensus_state_tracker::{CelestiaConfig, CelestiaService};

    /// Returns a Celestia DA service that can be used for testing.
    pub async fn celestia_da_service() -> CelestiaService {
        /// The rollup stores its data in the namespace b"sov-test" on Celestia
        /// You can change this constant to point your rollup at a different namespace
        pub const ROLLUP_BATCH_NAMESPACE: Namespace =
            Namespace::const_v0(ROLLUP_BATCH_NAMESPACE_RAW);

        /// The rollup stores the zk proofs in the namespace b"sov-test-p" on Celestia.
        pub const ROLLUP_PROOF_NAMESPACE: Namespace =
            Namespace::const_v0(ROLLUP_PROOF_NAMESPACE_RAW);

        let celestia_config: CelestiaConfig =
            crate::configs::from_toml_path("celestia_rollup_config.toml").unwrap();

        CelestiaService::new(
            celestia_config,
            RollupParams {
                rollup_batch_namespace: ROLLUP_BATCH_NAMESPACE,
                rollup_proof_namespace: ROLLUP_PROOF_NAMESPACE,
            },
        )
        .await
    }
}

#[cfg(feature = "celestia-da")]
pub use celestia_da::*;

#[cfg(feature = "mock-da")]
mod mock_da {
    use sov_consensus_state_tracker::{MockAddress, MockDaService};

    /// Returns a mock DA service that can be used for testing.
    pub fn mock_da_service() -> MockDaService {
        MockDaService::new(MockAddress::default())
    }
}

#[cfg(feature = "mock-da")]
pub use mock_da::*;
