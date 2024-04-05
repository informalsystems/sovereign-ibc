use ibc_client_wasm_types::{SUBJECT_PREFIX, SUBSTITUTE_PREFIX};
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;

/// The RecoveryPrefix field indicates whether the recovery mode has been
/// activated if there is an incoming `MigrateClientStore` message. If so, it
/// specifies the prefix key for either the subject or substitute storage index.
pub enum RecoveryPrefix {
    Subject,
    Substitute,
    None,
}

impl RecoveryPrefix {
    pub fn key(&self) -> &[u8] {
        match self {
            RecoveryPrefix::Subject => SUBJECT_PREFIX,
            RecoveryPrefix::Substitute => SUBSTITUTE_PREFIX,
            RecoveryPrefix::None => b"",
        }
    }
}

/// Travel is an enum to represent the direction of travel in the context of
/// height.
pub enum HeightTravel {
    Next,
    Prev,
}

/// Decodes a `Height` from a UTF-8 encoded byte array.
pub fn parse_height(encoded_height: Vec<u8>) -> Result<Height, ClientError> {
    let height_str =
        alloc::str::from_utf8(encoded_height.as_slice()).map_err(|e| ClientError::Other {
            description: e.to_string(),
        })?;

    Height::try_from(height_str).map_err(|e| ClientError::Other {
        description: e.to_string(),
    })
}
