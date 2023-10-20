use alloc::string::String;
use std::error::Error as StdError;

use derive_more::{Display, From};
use ibc::core::ics02_client::error::ClientError;
use ibc::core::ics23_commitment::error::CommitmentError;
use ibc::core::ics24_host::path::PathError;
use ibc::core::ContextError;
use tendermint_light_client_verifier::Verdict;

#[derive(From, Display, Debug)]
pub enum ContractError {
    Std(cosmwasm_std::StdError),
    #[display(fmt = "Celestia error: {_0}")]
    Celestia(String),
    #[display(fmt = "IBC validation/execution context error: {_0}")]
    Context(ContextError),
    #[display(fmt = "IBC path error: {_0}")]
    Path(PathError),
    #[display(fmt = "IBC commitment error: {_0}")]
    Commitment(CommitmentError),
    #[display(fmt = "Proto decode error: {_0}")]
    ProtoDecode(prost::DecodeError),
}

impl StdError for ContractError {}

pub(crate) trait IntoResult<T, E> {
    fn into_result(self) -> Result<T, E>;
}

impl IntoResult<(), ClientError> for Verdict {
    fn into_result(self) -> Result<(), ClientError> {
        match self {
            Verdict::Success => Ok(()),
            Verdict::NotEnoughTrust(reason) => Err(ClientError::Other {
                description: reason.to_string(),
            }),
            Verdict::Invalid(detail) => Err(ClientError::Other {
                description: detail.to_string(),
            }),
        }
    }
}
