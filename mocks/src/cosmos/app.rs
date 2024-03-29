//! Defines a mock Cosmos chain that includes the basecoin app and a mock
//! tendermint instance.
use std::fmt::Debug;

use basecoin::app::abci::v0_37::impls::query as basecoin_query;
use basecoin::app::{BaseCoinApp, Builder};
use basecoin::helper::{bank, ibc};
use basecoin::modules::auth::proto::AccountId;
use basecoin::modules::auth::Auth;
use basecoin::modules::bank::{Bank, BankReader, Denom};
use basecoin::modules::context::{prefix, Identifiable};
use basecoin::modules::ibc::{Ibc, IbcContext};
use basecoin::store::context::{ProvableStore, Store};
use basecoin::store::impls::RevertibleStore;
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::CommitmentProofBytes;
use ibc_core::host::types::identifiers::ChainId;
use tendermint::abci::request::Query as RequestQuery;
use tendermint::abci::response::Query as ResponseQuery;

use super::utils::convert_tm_to_ics_merkle_proof;
use super::MockTendermint;

#[derive(Clone)]
pub struct MockCosmosChain<S>
where
    S: ProvableStore + Debug,
{
    /// Tendermint core
    pub core: MockTendermint,
    /// Chain application
    pub app: BaseCoinApp<S>,
}

impl<S: ProvableStore + Default + Debug> MockCosmosChain<S> {
    /// Constructs a new mock cosmos chain instance.
    pub fn new(core: MockTendermint, store: S) -> Self {
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

        Self { core, app }
    }

    pub fn chain_id(&self) -> &ChainId {
        self.core.chain_id()
    }

    pub fn ibc_ctx(&self) -> IbcContext<RevertibleStore<S>> {
        ibc(self.app.clone()).ctx()
    }

    pub fn get_balance_of(&self, denom_str: &str, account: String) -> Option<u64> {
        let account_id: AccountId = account.parse().unwrap();

        let denom = Denom(denom_str.to_string());

        if let Some(coin) = bank(self.app.clone())
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

    /// Queries the chain for a given path and height.
    pub fn query(
        &self,
        data: Vec<u8>,
        path: String,
        height: Option<Height>,
    ) -> (Vec<u8>, CommitmentProofBytes) {
        // If no height is provided, use the current height of the chain.
        let revision_height = match height {
            Some(height) => height.revision_height(),
            None => self.app.store.current_height(),
        };

        let response: ResponseQuery = basecoin_query(
            &self.app,
            RequestQuery {
                data: data.into(),
                path,
                height: revision_height.try_into().unwrap(),
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
