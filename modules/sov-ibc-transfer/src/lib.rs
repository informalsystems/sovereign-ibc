pub mod call;
pub mod context;
mod genesis;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use sov_modules_api::{Context, Error, Module, ModuleInfo, StateMap, WorkingSet};

#[cfg(feature = "native")]
mod query;
#[cfg(feature = "native")]
pub use query::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferConfig {}

#[cfg_attr(feature = "native", derive(sov_modules_api::ModuleCallJsonSchema))]
#[derive(ModuleInfo, Clone)]
pub struct IbcTransfer<C: Context> {
    /// Address of the module.
    #[address]
    address: C::Address,

    /// Reference to the Bank module.
    #[module]
    bank: sov_bank::Bank<C>,

    /// Keeps track of the address of each token we minted by token denom.
    #[state]
    minted_tokens: StateMap<String, C::Address>,

    /// Keeps track of the address of each token we escrowed as a function of
    /// the token denom. We need this map because we have the token address
    /// information when escrowing the tokens (i.e. when someone calls a
    /// `send_transfer()`), but not when unescrowing tokens (i.e in a
    /// `recv_packet`), in which case the only information we have is the ICS 20
    /// denom, and amount. Given that every token that is unescrowed has been
    /// previously escrowed, our strategy to get the token address associated
    /// with a denom is
    /// 1. when tokens are escrowed, save the mapping `denom -> token address`
    /// 2. when tokens are unescrowed, lookup the token address by `denom`
    #[state]
    escrowed_tokens: StateMap<String, C::Address>,
}

impl<C: Context> Module for IbcTransfer<C> {
    type Context = C;

    type Config = TransferConfig;

    type CallMessage = ();

    fn genesis(&self, config: &Self::Config, working_set: &mut WorkingSet<C>) -> Result<(), Error> {
        Ok(self.init_module(config, working_set)?)
    }

    fn call(
        &self,
        _msg: Self::CallMessage,
        _context: &Self::Context,
        _working_set: &mut WorkingSet<C>,
    ) -> Result<sov_modules_api::CallResponse, Error> {
        Err(Error::ModuleError(anyhow!(
            "Cannot call sov-ibc-transfer; use sov-ibc instead"
        )))
    }
}

impl<C: Context> core::fmt::Debug for IbcTransfer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIXME: put real values here, or remove `Debug` requirement from router::Module
        f.debug_struct("Transfer")
            .field("address", &self.address)
            .finish()
    }
}
