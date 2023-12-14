//! Contains the implementation of the mock Cosmos chain runner.
use core::fmt::Debug;
use std::time::Duration;

use basecoin_app::modules::types::IdentifiedModule;
use basecoin_store::context::{ProvableStore, Store};
use basecoin_store::utils::SharedRwExt;
use ibc_core::host::types::identifiers::ChainId;
use tendermint::abci::request::InitChain;
use tendermint::block::Height as TmHeight;
use tendermint::v0_37::abci::Request as AbciRequest;
use tendermint::{AppHash, Time};
use tendermint_testgen::consensus::default_consensus_params;
use tendermint_testgen::light_block::TmLightBlock;
use tendermint_testgen::{Generator, Header, LightBlock, Validator};
use tokio::task::JoinHandle;
use tower::Service;
use tracing::debug;

use super::{genesis_app_state, MockCosmosChain};
use crate::cosmos::MutexUtil;
use crate::utils::wait_for_block;

impl<S: ProvableStore + Default + Debug> MockCosmosChain<S> {
    /// Generates a new light block for the chain with the given parameters.
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

    /// Grows the chain by one block.
    pub fn grow_blocks(&self) {
        let root_hash = self.app.store.root_hash();

        let app_hash = AppHash::try_from(root_hash).expect("invalid app hash");

        let mut blocks = self.blocks.acquire_mutex();

        let validators = self.validators.acquire_mutex();

        let height = blocks.len() as u64 + 1;

        debug!("cosmos: growing chain to height {}", height);

        let new_tm_light_block =
            Self::generate_block(&self.chain_id, height, Time::now(), &validators, app_hash);

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

    /// Begins a new block on the chain.
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

    /// Runs the chain in a separate thread.
    pub async fn run(&self) -> JoinHandle<()> {
        let chain = self.clone();

        let handle = self.runtime().spawn(async move {
            chain.init().await;

            loop {
                chain.begin_block().await;

                tokio::time::sleep(Duration::from_millis(200)).await;

                chain.commit().await;
            }
        });

        wait_for_block().await;

        handle
    }
}
