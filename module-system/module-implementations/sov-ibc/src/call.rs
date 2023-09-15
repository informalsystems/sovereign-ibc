use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use anyhow::{bail, Result};
use ibc::core::{dispatch, MsgEnvelope};
use sov_ibc_transfer::call::SDKTokenTransfer;
use sov_ibc_transfer::context::TransferContext;
use sov_modules_api::CallResponse;
use sov_state::WorkingSet;
use thiserror::Error;

use crate::context::IbcExecutionContext;
use crate::router::IbcRouter;
use crate::Ibc;

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    schemars(bound = "C::Address: ::schemars::JsonSchema", rename = "CallMessage")
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq)]
pub enum CallMessage<C: sov_modules_api::Context> {
    // Q: Should this be `Core(Any)` instead, so that we don't expose our `MsgEnvelope` to the rpc interface?
    // In that case though, the bytes in `Any` would probably be encoded with `hex`, which imposes big parsing
    // costs to the node.
    //
    // Or maybe we use a type local to this module that is very similar to `MsgEnvelope` and use that,
    // to enable ibc-rs to change `MsgEnvelope` in the future?
    Core(MsgEnvelope),

    Transfer(SDKTokenTransfer<C>),
}

/// Example of a custom error.
#[derive(Debug, Error)]
enum SetValueError {}

impl<C, Da> Ibc<C, Da>
where
    C: sov_modules_api::Context,
    Da: sov_modules_api::DaSpec,
{
    pub(crate) fn process_core_message(
        &self,
        msg: MsgEnvelope,
        context: &C,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<sov_modules_api::CallResponse> {
        let shared_working_set = Rc::new(RefCell::new(working_set));

        let mut execution_context = IbcExecutionContext {
            ibc: self,
            working_set: shared_working_set.clone(),
        };

        let mut router = IbcRouter::new(self, context, shared_working_set);

        match dispatch(&mut execution_context, &mut router, msg) {
            Ok(_) => Ok(CallResponse::default()),
            Err(e) => bail!(e.to_string()),
        }
    }

    pub(crate) fn transfer(
        &self,
        sdk_token_transfer: SDKTokenTransfer<C>,
        context: &C,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<sov_modules_api::CallResponse> {
        let shared_working_set = Rc::new(RefCell::new(working_set));
        let mut execution_context = IbcExecutionContext {
            ibc: self,
            working_set: shared_working_set.clone(),
        };

        let mut token_ctx =
            TransferContext::new(self.transfer.clone(), context, shared_working_set.clone());

        self.transfer.transfer(
            sdk_token_transfer,
            &mut execution_context,
            &mut token_ctx,
            shared_working_set,
        )
    }
}
