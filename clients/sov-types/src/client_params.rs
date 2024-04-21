use std::str::FromStr;
use std::time::Duration;

use ibc_core::client::types::error::{ClientError, UpgradeClientError};
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::CommitmentPrefix;
use ibc_core::primitives::proto::Protobuf;

use crate::aggregated_proof::{CodeCommitment, Root};
use crate::error::Error;
use crate::proto::SovereignClientParams as RawSovereignClientParams;

/// Defines the Sovereign SDK rollup-specific client parameters.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SovereignClientParams {
    /// The height of the DA layer at which the rollup initialized.
    pub genesis_da_height: Height,
    /// The genesis state root, which is unique to each rollup. Of course
    /// assuming an honest rollup that has not tampered with its software.
    pub genesis_state_root: Root,
    /// The code commitment of the rollup's software, which is the output
    /// commitment of the ZK circuit.
    pub code_commitment: CodeCommitment,
    /// The trusting period is the period in which headers can be verified.
    ///
    /// Note: During each update, the client verifies both the core header of DA
    /// and the aggregated proof simultaneously. When setting this period,
    /// consider both the DA layer and the rollup, ensuring it is the maximum
    /// acceptable duration. If the rollup should have a shorter trusting
    /// window, it dictates when the client expires.
    pub trusting_period: Duration,
    /// The frozen height indicates whether the client is frozen.
    ///
    /// Note: When frozen, `ibc-rs` sets the height to Height::new(0, 1),
    /// following the same logic as in `ibc-go`.
    pub frozen_height: Option<Height>,
    /// The latest height of the rollup, which corresponds to the height of the
    /// last update.
    ///
    /// Note: The `ibc-rs` requires the use of the `Height` type. Therefore,
    /// incoming height values, identified as `SlotNumber` in the Sovereign SDK
    /// system and of type `u64`, should be converted to `Height` for
    /// compatibility with `ibc-rs` implementation.
    pub latest_height: Height,
    /// The upgrade path is the path to the location on rollup where the
    /// upgraded client and consensus states are stored.
    pub upgrade_path: UpgradePath,
}

impl SovereignClientParams {
    pub fn new(
        genesis_da_height: Height,
        genesis_state_root: Root,
        code_commitment: CodeCommitment,
        trusting_period: Duration,
        frozen_height: Option<Height>,
        latest_height: Height,
        upgrade_path: UpgradePath,
    ) -> Self {
        Self {
            genesis_da_height,
            genesis_state_root,
            code_commitment,
            trusting_period,
            frozen_height,
            latest_height,
            upgrade_path,
        }
    }

    /// Returns `true` if the respective fields of two `SovereignClientParams`
    /// match for the client recovery process.
    pub fn check_on_recovery(&self, substitute: &Self) -> bool {
        self.genesis_da_height == substitute.genesis_da_height
            && self.genesis_state_root == substitute.genesis_state_root
            && self.code_commitment == substitute.code_commitment
            && self.upgrade_path == substitute.upgrade_path
    }

    /// Updates the `SovereignClientParams` on the client recovery process with
    /// the given substitute.
    pub fn update_on_recovery(self, substitute: Self) -> Self {
        Self {
            trusting_period: substitute.trusting_period,
            frozen_height: None,
            latest_height: substitute.latest_height,
            ..self
        }
    }

    /// Updates the respective fields of the `SovereignClientParams` on the
    /// client upgrade process with the given upgraded client parameters.
    pub fn update_on_upgrade(self, upgraded: Self) -> Self {
        Self {
            genesis_da_height: upgraded.genesis_da_height,
            code_commitment: upgraded.code_commitment,
            latest_height: upgraded.latest_height,
            frozen_height: None,
            upgrade_path: upgraded.upgrade_path,
            ..self
        }
    }
}

impl Protobuf<RawSovereignClientParams> for SovereignClientParams {}

impl TryFrom<RawSovereignClientParams> for SovereignClientParams {
    type Error = ClientError;

    fn try_from(raw: RawSovereignClientParams) -> Result<Self, Self::Error> {
        let genesis_da_height = raw
            .genesis_da_height
            .ok_or(Error::missing("genesis_da_height"))?
            .try_into()?;

        let genesis_state_root = raw.genesis_state_root.try_into()?;

        let code_commitment = raw
            .code_commitment
            .ok_or(Error::missing("code_commitment"))?
            .into();

        let trusting_period = raw
            .trusting_period
            .ok_or(Error::missing("trusting_period"))?
            .try_into()
            .map_err(|_| Error::invalid("trusting_period"))?;

        let frozen_height = raw.frozen_height.map(TryInto::try_into).transpose()?;

        let latest_height = raw
            .latest_height
            .ok_or(Error::missing("latest_height"))?
            .try_into()?;

        let upgrade_path = raw.upgrade_path.try_into()?;

        Ok(Self::new(
            genesis_da_height,
            genesis_state_root,
            code_commitment,
            trusting_period,
            frozen_height,
            latest_height,
            upgrade_path,
        ))
    }
}

impl From<SovereignClientParams> for RawSovereignClientParams {
    fn from(value: SovereignClientParams) -> Self {
        RawSovereignClientParams {
            genesis_state_root: value.genesis_state_root.into(),
            genesis_da_height: Some(value.genesis_da_height.into()),
            code_commitment: Some(value.code_commitment.into()),
            trusting_period: Some(value.trusting_period.into()),
            frozen_height: value.frozen_height.map(Into::into),
            latest_height: Some(value.latest_height.into()),
            upgrade_path: value.upgrade_path.0,
        }
    }
}

/// Defines the upgrade path type for the Sovereign SDK rollup.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct UpgradePath(String);

impl Default for UpgradePath {
    fn default() -> Self {
        Self("sov_ibc/Ibc/".to_string())
    }
}

impl UpgradePath {
    pub fn new(path: String) -> Self {
        Self(path)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for UpgradePath {
    type Error = ClientError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl FromStr for UpgradePath {
    type Err = ClientError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(UpgradeClientError::Other {
                reason: "empty upgrade path".into(),
            })?;
        }

        Ok(Self(s.to_string()))
    }
}

impl TryFrom<UpgradePath> for CommitmentPrefix {
    type Error = ClientError;

    fn try_from(value: UpgradePath) -> Result<Self, Self::Error> {
        CommitmentPrefix::try_from(value.0.into_bytes())
            .map_err(ClientError::InvalidCommitmentProof)
    }
}

#[cfg(feature = "test-util")]
pub mod test_util {

    use super::*;

    #[derive(typed_builder::TypedBuilder, Debug)]
    #[builder(build_method(into = SovereignClientParams))]
    pub struct SovereignParamsConfig {
        #[builder(default = Height::new(0, 3).unwrap())]
        pub genesis_da_height: Height,
        #[builder(default = Root::from([0; 32]))]
        pub genesis_state_root: Root,
        #[builder(default = CodeCommitment::from(vec![1; 32]))]
        pub code_commitment: CodeCommitment,
        #[builder(default = Duration::from_secs(64000))]
        pub trusting_period: Duration,
        #[builder(default)]
        pub frozen_height: Option<Height>,
        pub latest_height: Height,
        #[builder(default)]
        pub upgrade_path: UpgradePath,
    }

    impl From<SovereignParamsConfig> for SovereignClientParams {
        fn from(config: SovereignParamsConfig) -> Self {
            SovereignClientParams::new(
                config.genesis_da_height,
                config.genesis_state_root,
                config.code_commitment,
                config.trusting_period,
                config.frozen_height,
                config.latest_height,
                config.upgrade_path,
            )
        }
    }
}
