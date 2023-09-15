use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use basecoin_app::modules::auth::Auth;
use basecoin_app::modules::bank::Bank;
use basecoin_app::modules::context::{prefix, Identifiable};
use basecoin_app::modules::ibc::{Ibc, IbcContext};
use basecoin_app::modules::types::IdentifiedModule;
use basecoin_app::{BaseCoinApp, Builder};
use basecoin_store::context::{ProvableStore, Store};
use basecoin_store::impls::RevertibleStore;
use basecoin_store::utils::SharedRwExt;
use ibc::core::ics23_commitment::commitment::CommitmentProofBytes;
use ibc::core::ics24_host::identifier::ChainId;
use ibc::core::ics24_host::path::Path;
use ibc::hosts::tendermint::IBC_QUERY_PATH;
use ibc::Height;
use tendermint::abci::request::{InitChain, Query};
use tendermint::block::Height as TmHeight;
use tendermint::v0_37::abci::{Request as AbciRequest, Response as AbciResponse};
use tendermint::{AppHash, Time};
use tendermint_testgen::consensus::default_consensus_params;
use tendermint_testgen::light_block::TmLightBlock;
use tendermint_testgen::{Generator, Header, LightBlock, Validator};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tower::Service;

use super::helpers::{convert_tm_to_ics_merkle_proof, genesis_app_state, MutexUtil};

/// Defines a mock Cosmos chain that includes simplified store, application,
/// consensus layers.
#[derive(Clone)]
pub struct MockCosmosChain<S>
where
    S: ProvableStore + Debug,
{
    pub runtime: Arc<Runtime>,
    /// Chain identifier
    pub chain_id: ChainId,
    /// Chain validators
    pub validators: Arc<Mutex<Vec<Validator>>>,
    /// Chain blocks
    pub blocks: Arc<Mutex<Vec<TmLightBlock>>>,
    /// Chain application
    pub app: BaseCoinApp<S>,
}

impl<S: ProvableStore + Default + Debug> MockCosmosChain<S> {
    /// Constructs a new mock cosmos chain instance.
    pub fn new(
        runtime: Arc<Runtime>,
        chain_id: ChainId,
        validators: Vec<Validator>,
        store: S,
    ) -> Self {
        let app_builder = Builder::new(store);

        let auth = Auth::new(app_builder.module_store(&prefix::Auth {}.identifier()));
        let bank = Bank::new(
            app_builder.module_store(&prefix::Bank {}.identifier()),
            auth.account_reader().clone(),
            auth.account_keeper().clone(),
        );
        let ibc = Ibc::new(
            app_builder.module_store(&prefix::Ibc {}.identifier()),
            bank.bank_keeper().clone(),
        );

        // register modules with the app
        let app = app_builder
            .add_module(prefix::Auth {}.identifier(), auth)
            .add_module(prefix::Bank {}.identifier(), bank)
            .add_module(prefix::Ibc {}.identifier(), ibc)
            .build();

        let genesis_height = Height::new(chain_id.revision_number(), 1).expect("never fails");

        let genesis_time = Time::now();

        let genesis_block = Self::generate_block(
            &chain_id,
            genesis_height.revision_height(),
            genesis_time,
            &validators,
            AppHash::default(),
        );

        Self {
            runtime,
            chain_id,
            validators: Arc::new(Mutex::new(validators)),
            blocks: Arc::new(Mutex::new(vec![genesis_block])),
            app,
        }
    }

    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    pub fn ibc_ctx(&self) -> IbcContext<RevertibleStore<S>> {
        self.app.ibc().ctx()
    }

    pub fn get_blocks(&self) -> Vec<TmLightBlock> {
        self.blocks.acquire_mutex().clone()
    }

    pub fn generate_block(
        chain_id: &ChainId,
        height: u64,
        time: Time,
        validators: &[Validator],
        app_hash: AppHash,
    ) -> TmLightBlock {
        let header = Header::new(validators)
            .chain_id(&chain_id.to_string())
            .height(height)
            .time(time)
            .next_validators(validators)
            .app_hash(app_hash);

        LightBlock::new_default_with_header(header)
            .generate()
            .expect("failed to generate light block")
    }

    pub fn grow_blocks(&self) {
        let root_hash = self.app.store.root_hash();

        let app_hash = AppHash::try_from(root_hash).expect("invalid app hash");

        let mut blocks = self.blocks.acquire_mutex();

        let validators = self.validators.acquire_mutex();

        let new_tm_light_block = Self::generate_block(
            &self.chain_id,
            blocks.len() as u64 + 1,
            Time::now(),
            &validators,
            app_hash,
        );

        blocks.push(new_tm_light_block);
    }

    /// Initialize the chain with the app state.
    async fn init(&self) {
        let app_state = serde_json::to_vec(&genesis_app_state()).expect("infallible serialization");

        let request = InitChain {
            time: Time::now(),
            chain_id: self.chain_id.to_string(),
            consensus_params: default_consensus_params(),
            validators: vec![],
            app_state_bytes: app_state.into(),
            initial_height: TmHeight::from(1_u8),
        };

        let mut app = self.app.clone();

        app.call(AbciRequest::InitChain(request))
            .await
            .expect("failed to initialize chain");
    }

    async fn begin_block(&self) {
        self.grow_blocks();

        let last_block = self.blocks.acquire_mutex().last().unwrap().clone();

        let mut events = Vec::new();

        let mut modules = self.app.modules.write_access();

        for IdentifiedModule { id: _, module } in modules.iter_mut() {
            let event = module.begin_block(&last_block.signed_header.header);
            events.extend(event);
        }
    }

    /// Commits the chain state to the store.
    async fn commit(&self) {
        let mut modules = self.app.modules.write_access();

        let mut state = self.app.store.write_access();

        for IdentifiedModule { id, module } in modules.iter_mut() {
            module
                .store_mut()
                .commit()
                .expect("failed to commit to state");

            state
                .set(id.clone().into(), module.store().root_hash())
                .expect("failed to update sub-store commitment");
        }

        state.commit().expect("failed to commit to state");
    }

    pub fn run(&self) -> JoinHandle<()> {
        let chain = self.clone();

        self.runtime().spawn(async move {
            chain.init().await;

            loop {
                chain.begin_block().await;

                tokio::time::sleep(Duration::from_millis(200)).await;

                chain.commit().await;
            }
        })
    }

    pub async fn query(
        &self,
        path: impl Into<Path> + Send,
        height: &Height,
    ) -> (Vec<u8>, CommitmentProofBytes) {
        let request = Query {
            path: IBC_QUERY_PATH.to_string(),
            data: path.into().to_string().into_bytes().into(),
            height: TmHeight::try_from(height.revision_height()).unwrap(),
            prove: true,
        };

        let mut app = self.app.clone();

        let response = match app.call(AbciRequest::Query(request)).await.unwrap() {
            AbciResponse::Query(res) => res,
            _ => panic!("unexpected response from query"),
        };

        let proof = match response.proof {
            Some(proof) => proof,
            None => panic!("proof not found in query response"),
        };

        let merkle_proof = convert_tm_to_ics_merkle_proof(&proof);

        let commitment_proof = merkle_proof.try_into().unwrap();

        (response.value.into(), commitment_proof)
    }
}
