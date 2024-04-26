pub mod context;
mod genesis;
pub mod utils;

use anyhow::anyhow;
use ibc_core::handler::types::events::IbcEvent;
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use serde::{Deserialize, Serialize};
use sov_bank::TokenId;
use sov_modules_api::{Context, Error, Module, ModuleId, ModuleInfo, Spec, StateMap, WorkingSet};

#[cfg(feature = "native")]
mod rpc;
#[cfg(feature = "native")]
pub use rpc::*;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct TransferConfig {}

#[cfg_attr(feature = "native", derive(sov_modules_api::ModuleCallJsonSchema))]
#[derive(ModuleInfo, Clone)]
pub struct IbcTransfer<S: Spec> {
    /// Id of the module.
    #[id]
    pub id: ModuleId,

    /// Reference to the Bank module.
    #[module]
    bank: sov_bank::Bank<S>,

    /// Maps the token name to its corresponding token ID for tokens minted
    /// through IBC. This mapping is used during mint/burn processes to validate
    /// if the token exists and gives out needed ID for the minting/burning
    /// process.
    #[state]
    minted_token_name_to_id: StateMap<String, TokenId>,

    /// Maps the token ID to its corresponding token name for tokens minted
    /// through IBC. This mapping is used during escrows/un-escrows to verify if
    /// the `TokenId` extracted from the `denom` is not an IBC-minted token,
    /// indicating it is a native token for escrow/un-escrows purposes.
    #[state]
    minted_token_id_to_name: StateMap<TokenId, String>,

    /// Keeps track of escrow addresses associated with a specific port and
    /// channel pair, offering an efficient means to access these addresses
    /// without the need for recomputation during every packet processing.
    #[state]
    escrow_address_cache: StateMap<(PortId, ChannelId), S::Address>,
}

impl<S: Spec> Module for IbcTransfer<S> {
    type Spec = S;

    type Config = TransferConfig;

    type CallMessage = ();

    type Event = IbcEvent;

    fn genesis(&self, config: &Self::Config, working_set: &mut WorkingSet<S>) -> Result<(), Error> {
        Ok(self.init_module(config, working_set)?)
    }

    fn call(
        &self,
        _msg: Self::CallMessage,
        _context: &Context<Self::Spec>,
        _working_set: &mut WorkingSet<S>,
    ) -> Result<sov_modules_api::CallResponse, Error> {
        Err(Error::ModuleError(anyhow!(
            "Cannot call sov-ibc-transfer; use sov-ibc instead"
        )))
    }
}

impl<S: Spec> core::fmt::Debug for IbcTransfer<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIXME: put real values here, or remove `Debug` requirement from router::Module
        f.debug_struct("Transfer").field("id", &self.id).finish()
    }
}
