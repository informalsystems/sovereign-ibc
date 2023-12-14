use const_rollup_config::{ROLLUP_BATCH_NAMESPACE_RAW, ROLLUP_PROOF_NAMESPACE_RAW};
use sov_celestia_adapter::types::Namespace;
use sov_celestia_adapter::verifier::RollupParams;
use sov_celestia_adapter::{CelestiaConfig, CelestiaService};

pub async fn celestia_da_service() -> CelestiaService {
    /// The rollup stores its data in the namespace b"sov-test" on Celestia
    /// You can change this constant to point your rollup at a different namespace
    pub const ROLLUP_BATCH_NAMESPACE: Namespace = Namespace::const_v0(ROLLUP_BATCH_NAMESPACE_RAW);

    /// The rollup stores the zk proofs in the namespace b"sov-test-p" on Celestia.
    pub const ROLLUP_PROOF_NAMESPACE: Namespace = Namespace::const_v0(ROLLUP_PROOF_NAMESPACE_RAW);

    let celestia_config = CelestiaConfig {
        celestia_rpc_auth_token: "".to_string(),

        celestia_rpc_address: "http://127.0.0.1:26658".to_string(),

        max_celestia_response_body_size: 104_857_600,

        celestia_rpc_timeout_seconds: 60,
    };

    CelestiaService::new(
        celestia_config,
        RollupParams {
            rollup_batch_namespace: ROLLUP_BATCH_NAMESPACE,
            rollup_proof_namespace: ROLLUP_PROOF_NAMESPACE,
        },
    )
    .await
}
