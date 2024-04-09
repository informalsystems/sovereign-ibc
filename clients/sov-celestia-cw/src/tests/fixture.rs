use std::ops::Add;
use std::time::Duration;

use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{coins, from_json, Deps, DepsMut, Empty, MessageInfo, StdError};
use ibc_client_tendermint::types::Header;
use ibc_core::client::types::{Height, Status};
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::primitives::Timestamp;
use sov_celestia_client::types::client_message::test_util::dummy_sov_header;
use sov_celestia_client::types::client_message::{Root, SovTmHeader};
use sov_celestia_client::types::client_state::test_util::{
    dummy_checksum, dummy_sov_consensus_state, mock_celestia_chain_id, ClientStateConfig,
    TendermintParamsConfig,
};
use sov_celestia_client::types::client_state::SovTmClientState;
use sov_celestia_client::types::codec::AnyCodec;
use sov_celestia_client::types::consensus_state::SovTmConsensusState;
use tendermint_testgen::{Generator, Validator};

use crate::entrypoints::SovTmContext;
use crate::types::{
    CheckForMisbehaviourMsgRaw, ExportMetadataMsg, GenesisMetadata, InstantiateMsg, QueryMsg,
    QueryResponse, StatusMsg, VerifyClientMessageRaw,
};

/// Test fixture
#[derive(Clone, Debug)]
pub struct Fixture {
    pub chain_id: ChainId,
    pub trusted_timestamp: Timestamp,
    pub trusted_height: Height,
    pub target_height: Height,
    pub validators: Vec<Validator>,
    pub migration_mode: bool,
}

impl Default for Fixture {
    fn default() -> Self {
        Fixture {
            chain_id: ChainId::new("rollup").unwrap(),
            // Returns a dummy timestamp for testing purposes. The value corresponds to the
            // timestamp of the `mock_env()`.
            trusted_timestamp: Timestamp::from_nanoseconds(1_571_797_419_879_305_533)
                .expect("never fails"),
            trusted_height: Height::new(0, 5).unwrap(),
            target_height: Height::new(0, 10).unwrap(),
            validators: vec![
                Validator::new("1").voting_power(40),
                Validator::new("2").voting_power(30),
                Validator::new("3").voting_power(30),
            ],
            migration_mode: false,
        }
    }
}

impl Fixture {
    pub fn migration_mode(mut self) -> Self {
        self.migration_mode = true;
        self
    }

    pub fn ctx_ref<'a>(&self, deps: Deps<'a, Empty>) -> SovTmContext<'a> {
        let mut ctx = SovTmContext::new_ref(deps, mock_env()).expect("never fails");

        if self.migration_mode {
            ctx.set_subject_prefix();
        };

        ctx
    }

    pub fn ctx_mut<'a>(&self, deps: DepsMut<'a, Empty>) -> SovTmContext<'a> {
        let mut ctx = SovTmContext::new_mut(deps, mock_env()).expect("never fails");

        if self.migration_mode {
            ctx.set_subject_prefix();
        };

        ctx
    }

    pub fn dummy_instantiate_msg(&self) -> InstantiateMsg {
        // Setting the `trusting_period`` to 1 second allows for a quick client
        // freeze in the `happy_cw_client_recovery` test
        let tendermint_params = TendermintParamsConfig::builder()
            .trusting_period(Duration::from_secs(1))
            .build();

        let sov_client_state = ClientStateConfig::builder()
            .latest_height(self.trusted_height)
            .tendermint_params(tendermint_params)
            .build();

        let sov_consensus_state = dummy_sov_consensus_state(self.trusted_timestamp);

        InstantiateMsg {
            client_state: SovTmClientState::encode_thru_any(sov_client_state),
            consensus_state: SovTmConsensusState::encode_thru_any(sov_consensus_state),
            checksum: dummy_checksum(),
        }
    }

    fn dummy_header(&self, header_height: Height) -> Vec<u8> {
        // NOTE: since mock context has a fixed timestamp, we only can add up
        // to allowed clock drift (3s)
        let future_time = self
            .trusted_timestamp
            .add(Duration::from_secs(2))
            .expect("never fails")
            .into_tm_time()
            .expect("Time exists");

        let header = tendermint_testgen::Header::new(&self.validators)
            .chain_id(mock_celestia_chain_id().as_str())
            .height(header_height.revision_height())
            .time(future_time)
            .next_validators(&self.validators)
            .app_hash(vec![0; 32].try_into().expect("never fails"));

        let light_block = tendermint_testgen::LightBlock::new_default_with_header(header)
            .generate()
            .expect("failed to generate light block");

        let tm_header = Header {
            signed_header: light_block.signed_header,
            validator_set: light_block.validators,
            trusted_height: self.trusted_height,
            trusted_next_validator_set: light_block.next_validators,
        };

        let sov_header = dummy_sov_header(
            tm_header,
            self.trusted_height,
            header_height,
            Root::from([0; 32]),
        );

        SovTmHeader::encode_thru_any(sov_header)
    }

    pub fn dummy_client_message(&self) -> Vec<u8> {
        self.dummy_header(self.target_height)
    }

    /// Constructs a dummy misbehaviour message that is one block behind the
    /// trusted height, but with a future timestamp.
    pub fn dummy_misbehaviour_message(&self) -> Vec<u8> {
        let prev_height = self.trusted_height.decrement().expect("never fails");

        self.dummy_header(prev_height)
    }

    pub fn verify_client_message(&self, deps: Deps<'_>, client_message: Vec<u8>) {
        let resp = self.query(deps, VerifyClientMessageRaw { client_message }.into());

        assert!(resp.is_valid);
        assert!(resp.status.is_none());
        assert!(resp.found_misbehaviour.is_none());
    }

    pub fn check_for_misbehaviour(&self, deps: Deps<'_>, client_message: Vec<u8>) {
        let resp = self.query(deps, CheckForMisbehaviourMsgRaw { client_message }.into());

        assert!(resp.is_valid);
        assert_eq!(resp.found_misbehaviour, Some(true));
    }

    pub fn check_client_status(&self, deps: Deps<'_>, expected: Status) {
        let resp = self.query(deps, StatusMsg {}.into());

        assert_eq!(resp.status, Some(expected.to_string()));
    }

    pub fn get_metadata(&self, deps: Deps<'_>) -> Option<Vec<GenesisMetadata>> {
        self.query(deps, ExportMetadataMsg {}.into())
            .genesis_metadata
    }

    pub fn query(&self, deps: Deps<'_>, msg: QueryMsg) -> QueryResponse {
        let ctx = self.ctx_ref(deps);

        let resp_bytes = ctx
            .query(msg)
            .map_err(|e| StdError::generic_err(e.to_string()))
            .unwrap();

        from_json(resp_bytes).unwrap()
    }
}

pub fn dummy_msg_info() -> MessageInfo {
    mock_info("creator", &coins(1000, "ibc"))
}
