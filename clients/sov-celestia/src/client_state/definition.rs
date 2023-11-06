use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::max;
use core::time::Duration;

use ibc::clients::ics07_tendermint::trust_threshold::TrustThreshold;
use ibc::core::ics02_client::error::ClientError;
use ibc::core::ics23_commitment::specs::ProofSpecs;
use ibc::core::ics24_host::identifier::ChainId;
use ibc::core::timestamp::ZERO_DURATION;
use ibc::proto::Any;
use ibc::Height;
use ibc_proto::ibc::core::client::v1::Height as RawHeight;
use prost::Message;
use tendermint::chain::id::MAX_LENGTH as MaxChainIdLen;
use tendermint::trust_threshold::{
    TrustThresholdFraction as TendermintTrustThresholdFraction, TrustThresholdFraction,
};
use tendermint_light_client_verifier::options::Options;
use tendermint_light_client_verifier::ProdVerifier;
use tendermint_proto::Protobuf;

use crate::client_message::SovHeader;
use crate::error::Error;
use crate::proto::ClientState as RawSovClientState;

pub const SOVEREIGN_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.sovereign.v1.ClientState";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AllowUpdate {
    pub after_expiry: bool,
    pub after_misbehaviour: bool,
}

/// Contains the core implementation of the Sovereign light client
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ClientState {
    pub chain_id: ChainId,
    pub trust_level: TrustThreshold,
    pub trusting_period: Duration,
    pub unbonding_period: Duration,
    pub max_clock_drift: Duration,
    pub latest_height: Height,
    pub proof_specs: ProofSpecs,
    pub upgrade_path: Vec<String>,
    pub allow_update: AllowUpdate,
    pub frozen_height: Option<Height>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub da_verifier: ProdVerifier,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub snark_verifier: String,
}

impl ClientState {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_without_validation(
        chain_id: ChainId,
        trust_level: TrustThreshold,
        trusting_period: Duration,
        unbonding_period: Duration,
        max_clock_drift: Duration,
        latest_height: Height,
        proof_specs: ProofSpecs,
        upgrade_path: Vec<String>,
        allow_update: AllowUpdate,
    ) -> Self {
        Self {
            chain_id,
            trust_level,
            trusting_period,
            unbonding_period,
            max_clock_drift,
            latest_height,
            proof_specs,
            upgrade_path,
            allow_update,
            frozen_height: None,
            da_verifier: ProdVerifier::default(),
            snark_verifier: String::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chain_id: ChainId,
        trust_level: TrustThreshold,
        trusting_period: Duration,
        unbonding_period: Duration,
        max_clock_drift: Duration,
        latest_height: Height,
        proof_specs: ProofSpecs,
        upgrade_path: Vec<String>,
        allow_update: AllowUpdate,
    ) -> Result<Self, Error> {
        let client_state = Self::new_without_validation(
            chain_id,
            trust_level,
            trusting_period,
            unbonding_period,
            max_clock_drift,
            latest_height,
            proof_specs,
            upgrade_path,
            allow_update,
        );
        client_state.validate()?;
        Ok(client_state)
    }

    pub fn with_header(self, header: SovHeader) -> Result<Self, ClientError> {
        Ok(Self {
            latest_height: max(header.height(), self.latest_height),
            ..self
        })
    }

    pub fn with_frozen_height(self, h: Height) -> Self {
        Self {
            frozen_height: Some(h),
            ..self
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        self.chain_id.validate_length(3, MaxChainIdLen as u64)?;

        // `TrustThreshold` is guaranteed to be in the range `[0, 1)`, but a `TrustThreshold::ZERO`
        // value is invalid in this context
        if self.trust_level == TrustThreshold::ZERO {
            return Err(Error::invalid("ClientState trust-level cannot be zero"));
        }

        TendermintTrustThresholdFraction::new(
            self.trust_level.numerator(),
            self.trust_level.denominator(),
        )
        .map_err(Error::source)?;

        // Basic validation of trusting period and unbonding period: each should be non-zero.
        if self.trusting_period <= Duration::new(0, 0) {
            return Err(Error::invalid(format!(
                "ClientState trusting period ({:?}) must be greater than zero",
                self.trusting_period
            )));
        }

        if self.unbonding_period <= Duration::new(0, 0) {
            return Err(Error::invalid(format!(
                "ClientState unbonding period ({:?}) must be greater than zero",
                self.unbonding_period
            )));
        }

        if self.trusting_period >= self.unbonding_period {
            return Err(Error::invalid(format!(
                "ClientState trusting period ({:?}) must be smaller than unbonding period ({:?})",
                self.trusting_period, self.unbonding_period
            )));
        }

        if self.max_clock_drift <= Duration::new(0, 0) {
            return Err(Error::invalid(format!(
                "ClientState max clock drift ({:?}) must be greater than zero",
                self.max_clock_drift
            )));
        }

        if self.latest_height.revision_number() != self.chain_id.revision_number() {
            return Err(Error::invalid(
                "ClientState latest height revision number must match chain-id revision number",
            ));
        }

        // Disallow empty proof-specs
        if self.proof_specs.is_empty() {
            return Err(Error::empty("ClientState proof-specs"));
        }

        // `upgrade_path` itself may be empty, but if not then each key must be non-empty
        for (idx, key) in self.upgrade_path.iter().enumerate() {
            if key.trim().is_empty() {
                return Err(Error::empty(format!(
                    "ClientState upgrade-path key at index {idx:?}"
                )));
            }
        }

        Ok(())
    }

    /// Get the refresh time to ensure the state does not expire
    pub fn refresh_time(&self) -> Option<Duration> {
        Some(2 * self.trusting_period / 3)
    }

    /// Helper method to produce a [`Options`] struct for use in
    /// Celestia-specific light client verification.
    pub fn as_light_client_options(&self) -> Result<Options, ClientError> {
        let trust_threshold = TrustThresholdFraction::new(
            self.trust_level.numerator(),
            self.trust_level.denominator(),
        )
        .map_err(|e| ClientError::Other {
            description: e.to_string(),
        })?;
        Ok(Options {
            trust_threshold,
            trusting_period: self.trusting_period,
            clock_drift: self.max_clock_drift,
        })
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id.clone()
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen_height.is_some()
    }

    // Resets custom fields to zero values (used in `update_client`)
    pub fn zero_custom_fields(&mut self) {
        self.trusting_period = ZERO_DURATION;
        self.trust_level = TrustThreshold::ZERO;
        self.allow_update.after_expiry = false;
        self.allow_update.after_misbehaviour = false;
        self.frozen_height = None;
        self.max_clock_drift = ZERO_DURATION;
    }
}

impl From<ClientState> for RawSovClientState {
    fn from(value: ClientState) -> Self {
        #[allow(deprecated)]
        Self {
            chain_id: value.chain_id.to_string(),
            trust_level: Some(value.trust_level.into()),
            trusting_period: Some(value.trusting_period.into()),
            unbonding_period: Some(value.unbonding_period.into()),
            max_clock_drift: Some(value.max_clock_drift.into()),
            frozen_height: Some(value.frozen_height.map(|height| height.into()).unwrap_or(
                RawHeight {
                    revision_number: 0,
                    revision_height: 0,
                },
            )),
            latest_height: Some(value.latest_height.into()),
            proof_specs: value.proof_specs.into(),
            upgrade_path: value.upgrade_path,
            allow_update_after_expiry: value.allow_update.after_expiry,
            allow_update_after_misbehaviour: value.allow_update.after_misbehaviour,
        }
    }
}

impl Protobuf<Any> for ClientState {}

impl TryFrom<Any> for ClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        use core::ops::Deref;

        use bytes::Buf;

        fn decode_client_state<B: Buf>(buf: B) -> Result<ClientState, ClientError> {
            RawSovClientState::decode(buf)
                .map_err(ClientError::Decode)?
                .try_into()
                .map_err(|e: Error| ClientError::Other {
                    description: e.to_string(),
                })
        }

        match raw.type_url.as_str() {
            SOVEREIGN_CLIENT_STATE_TYPE_URL => {
                decode_client_state(raw.value.deref()).map_err(Into::into)
            }
            _ => Err(ClientError::UnknownClientStateType {
                client_state_type: raw.type_url,
            }),
        }
    }
}

impl From<ClientState> for Any {
    fn from(client_state: ClientState) -> Self {
        Any {
            type_url: SOVEREIGN_CLIENT_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawSovClientState>::encode_vec(&client_state).unwrap(),
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_util {
    use ics08_wasm::client_state::ClientState as WasmClientState;

    use super::*;
    use crate::Bytes;

    #[derive(typed_builder::TypedBuilder, Debug)]
    pub struct ClientStateConfig {
        pub chain_id: ChainId,
        #[builder(default)]
        pub trust_level: TrustThreshold,
        #[builder(default = Duration::from_secs(64000))]
        pub trusting_period: Duration,
        #[builder(default = Duration::from_secs(128000))]
        pub unbonding_period: Duration,
        #[builder(default = Duration::from_millis(3000))]
        max_clock_drift: Duration,
        pub latest_height: Height,
        #[builder(default)]
        pub proof_specs: ProofSpecs,
        #[builder(default)]
        pub upgrade_path: Vec<String>,
        #[builder(default = AllowUpdate { after_expiry: false, after_misbehaviour: false })]
        allow_update: AllowUpdate,
    }

    impl TryFrom<ClientStateConfig> for ClientState {
        type Error = Error;

        fn try_from(config: ClientStateConfig) -> Result<Self, Self::Error> {
            ClientState::new(
                config.chain_id,
                config.trust_level,
                config.trusting_period,
                config.unbonding_period,
                config.max_clock_drift,
                config.latest_height,
                config.proof_specs,
                config.upgrade_path,
                config.allow_update,
            )
        }
    }

    impl ClientState {
        pub fn into_wasm(&self, code_hash: Bytes) -> WasmClientState {
            WasmClientState {
                data: Any::from(self.clone()).value,
                code_hash,
                latest_height: self.latest_height,
            }
        }
    }
}
