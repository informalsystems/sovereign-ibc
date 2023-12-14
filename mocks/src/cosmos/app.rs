//! Defines a mock Cosmos chain that includes simplified store, application,
//! consensus layers.
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use basecoin_app::abci::v0_37::impls::query as basecoin_query;
use basecoin_app::modules::auth::proto::AccountId;
use basecoin_app::modules::auth::Auth;
use basecoin_app::modules::bank::{Bank, BankReader, Denom};
use basecoin_app::modules::context::{prefix, Identifiable};
use basecoin_app::modules::ibc::{Ibc, IbcContext};
use basecoin_app::{BaseCoinApp, Builder};
use basecoin_store::context::ProvableStore;
use basecoin_store::impls::RevertibleStore;
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::CommitmentProofBytes;
use ibc_core::host::types::identifiers::ChainId;
use tendermint::abci::request::Query as RequestQuery;
use tendermint::abci::response::Query as ResponseQuery;
use tendermint::{AppHash, Time};
use tendermint_testgen::light_block::TmLightBlock;
use tendermint_testgen::Validator;
use tokio::runtime::Runtime;

use super::helpers::{convert_tm_to_ics_merkle_proof, MutexUtil};

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

        let genesis_block = Self::generate_block(
            &chain_id,
            genesis_height.revision_height(),
            Time::now(),
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

    pub fn get_balance_of(&self, denom: &str, account: String) -> Option<u64> {
        let account_id: AccountId = account.parse().unwrap();

        let denom = Denom(denom.to_string());

        if let Some(coin) = self
            .app
            .bank()
            .balance_reader()
            .get_all_balances(account_id)
            .into_iter()
            .find(|c| c.denom == denom)
        {
            Some(coin.amount.try_into().ok()?)
        } else {
            None
        }
    }

    pub fn get_blocks(&self) -> Vec<TmLightBlock> {
        self.blocks.acquire_mutex().clone()
    }

    /// Queries the chain for a given path and height.
    pub fn query(
        &self,
        data: Vec<u8>,
        path: String,
        height: &Height,
    ) -> (Vec<u8>, CommitmentProofBytes) {
        let response: ResponseQuery = basecoin_query(
            &self.app,
            RequestQuery {
                data: data.into(),
                path,
                height: height.revision_height().try_into().unwrap(),
                prove: true,
            }
            .into(),
        )
        .try_into()
        .unwrap();

        let proof = match response.proof {
            Some(proof) => proof,
            None => panic!("proof not found in query response"),
        };

        let merkle_proof = convert_tm_to_ics_merkle_proof(&proof);

        let commitment_proof = merkle_proof.try_into().unwrap();

        (response.value.into(), commitment_proof)
    }
}
