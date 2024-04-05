use std::ops::Add;
use std::time::Duration;

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_json, Deps, DepsMut, Empty, MessageInfo, StdError};
use ibc_client_tendermint::types::Header;
use ibc_core::client::types::{Height, Status};
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::primitives::Timestamp;
use sov_celestia_client::types::client_message::test_util::dummy_sov_header;
use sov_celestia_client::types::client_message::{Root, SovTmHeader};
use sov_celestia_client::types::client_state::test_util::{
    dummy_checksum, dummy_sov_client_state, dummy_sov_consensus_state, mock_celestia_chain_id,
};
use sov_celestia_client::types::client_state::SovTmClientState;
use sov_celestia_client::types::codec::AnyCodec;
use sov_celestia_client::types::consensus_state::SovTmConsensusState;
use tendermint_testgen::{Generator, Validator};

use crate::entrypoints::{instantiate, sudo, SovTmContext};
use crate::types::{
    CheckForMisbehaviourMsgRaw, ContractResult, ExportMetadataMsg, GenesisMetadata, InstantiateMsg,
    MigrateClientStoreMsg, QueryMsg, QueryResponse, StatusMsg, UpdateStateMsgRaw,
    UpdateStateOnMisbehaviourMsgRaw, VerifyClientMessageRaw,
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
        let sov_client_state = dummy_sov_client_state(self.chain_id.clone(), self.trusted_height);

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

#[test]
fn happy_cw_create_client() {
    let fxt = Fixture::default();

    let mut deps = mock_dependencies();

    let instantiate_msg = fxt.dummy_instantiate_msg();

    let resp = instantiate(deps.as_mut(), mock_env(), dummy_msg_info(), instantiate_msg).unwrap();

    assert_eq!(0, resp.messages.len());

    let contract_result: ContractResult = from_json(resp.data.unwrap()).unwrap();

    assert!(contract_result.heights.is_none());

    fxt.check_client_status(deps.as_ref(), Status::Active);
}

#[test]
fn happy_cw_update_client() {
    let fxt = Fixture::default();

    let mut deps = mock_dependencies();

    // ------------------- Create client -------------------

    let instantiate_msg = fxt.dummy_instantiate_msg();

    instantiate(deps.as_mut(), mock_env(), dummy_msg_info(), instantiate_msg).unwrap();

    // ------------------- Verify and Update client -------------------

    let client_message = fxt.dummy_client_message();

    fxt.verify_client_message(deps.as_ref(), client_message.clone());

    let resp = sudo(
        deps.as_mut(),
        mock_env(),
        UpdateStateMsgRaw { client_message }.into(),
    )
    .unwrap();

    // ------------------- Check response -------------------

    assert_eq!(0, resp.messages.len());

    let contract_result: ContractResult = from_json(resp.data.unwrap()).unwrap();

    assert_eq!(contract_result.heights, Some(vec![fxt.target_height]));

    fxt.check_client_status(deps.as_ref(), Status::Active);
}

#[test]
fn happy_cw_client_recovery() {
    let fxt = Fixture::default().migration_mode();

    let mut deps = mock_dependencies();

    let mut ctx = fxt.ctx_mut(deps.as_mut());

    // ------------------- Create subject client -------------------

    let instantiate_msg = fxt.dummy_instantiate_msg();

    let data = ctx.instantiate(instantiate_msg.clone()).unwrap();

    // ------------------- Freeze subject client -------------------

    let client_message = fxt.dummy_misbehaviour_message();

    fxt.check_for_misbehaviour(deps.as_ref(), client_message.clone());

    let mut ctx = fxt.ctx_mut(deps.as_mut());

    let data = ctx
        .sudo(UpdateStateOnMisbehaviourMsgRaw { client_message }.into())
        .unwrap();

    // ------------------- Create substitute client -------------------

    ctx.set_substitute_prefix();

    let data = ctx.instantiate(instantiate_msg).unwrap();

    // ------------------- Recover subject client -------------------

    let resp = sudo(deps.as_mut(), mock_env(), MigrateClientStoreMsg {}.into()).unwrap();

    assert_eq!(0, resp.messages.len());

    fxt.check_client_status(deps.as_ref(), Status::Active);
}
