//! Contains the implementation of the mock Cosmos chain runner.
use core::fmt::Debug;
use std::time::Duration;

use basecoin::modules::types::IdentifiedModule;
use basecoin::store::context::{ProvableStore, Store};
use basecoin::store::utils::SharedRwExt;
use tendermint::abci::request::InitChain;
use tendermint::block::Height as TmHeight;
use tendermint::v0_37::abci::Request as AbciRequest;
use tendermint::Time;
use tendermint_testgen::consensus::default_consensus_params;
use tokio::task::JoinHandle;
use tower::Service;

use super::{genesis_app_state, MockCosmosChain};
use crate::utils::wait_for_block;

impl<S: ProvableStore + Default + Debug> MockCosmosChain<S> {
    /// Initialize the chain with the app state.
    async fn init(&self) {
        let app_state = serde_json::to_vec(&genesis_app_state()).expect("infallible serialization");

        let request = InitChain {
            time: Time::now(),
            chain_id: self.chain_id().to_string(),
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
        self.core.grow_blocks(self.app.store.root_hash());

        let last_block = self.core.blocks().last().unwrap().clone();

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

        let handle = self.core.runtime().spawn(async move {
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
