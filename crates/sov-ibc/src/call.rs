use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use anyhow::{bail, Result};
use ibc::core::{dispatch, MsgEnvelope};
use ibc::proto::Any;
use sov_modules_api::{CallResponse, Context, DaSpec, WorkingSet};
use thiserror::Error;

use crate::applications::transfer::call::SDKTokenTransfer;
use crate::applications::transfer::context::TransferContext;
use crate::context::IbcExecutionContext;
use crate::router::IbcRouter;
use crate::Ibc;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    schemars(bound = "C::Address: ::schemars::JsonSchema", rename = "CallMessage")
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq)]
pub enum CallMessage<C: sov_modules_api::Context> {
    #[cfg_attr(feature = "native", schemars(with = "ibc::utils::schema::AnySchema"))]
    Core(Any),

    Transfer(SDKTokenTransfer<C>),
}

/// Example of a custom error.
#[derive(Debug, Error)]
enum SetValueError {}

impl<C: Context, Da: DaSpec> Ibc<C, Da> {
    pub(crate) fn process_core_message(
        &self,
        msg: Any,
        context: &C,
        working_set: &mut WorkingSet<C>,
    ) -> Result<sov_modules_api::CallResponse> {
        let shared_working_set = Rc::new(RefCell::new(working_set));

        let mut execution_context = IbcExecutionContext {
            ibc: self,
            working_set: shared_working_set.clone(),
        };

        let mut router = IbcRouter::new(self, context, shared_working_set);

        let msg_envelope = MsgEnvelope::try_from(msg).unwrap();

        match dispatch(&mut execution_context, &mut router, msg_envelope) {
            Ok(_) => Ok(CallResponse::default()),
            Err(e) => bail!(e.to_string()),
        }
    }

    pub(crate) fn transfer(
        &self,
        sdk_token_transfer: SDKTokenTransfer<C>,
        context: &C,
        working_set: &mut WorkingSet<C>,
    ) -> Result<CallResponse> {
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
