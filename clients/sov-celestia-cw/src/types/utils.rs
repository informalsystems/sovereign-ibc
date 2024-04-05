use ibc_client_wasm_types::{SUBJECT_PREFIX, SUBSTITUTE_PREFIX};
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;

/// The MigrationPrefix field indicates whether the recovery mode has been
/// activated if there is an incoming `MigrateClientStore` message. If so, it
/// specifies the prefix key for either the subject or substitute storage index.
#[derive(Clone, Debug)]
pub enum MigrationPrefix {
    Subject,
    Substitute,
    None,
}

impl MigrationPrefix {
    pub fn key(&self) -> &[u8] {
        match self {
            MigrationPrefix::Subject => SUBJECT_PREFIX,
            MigrationPrefix::Substitute => SUBSTITUTE_PREFIX,
            MigrationPrefix::None => b"",
        }
    }
}

/// Travel is an enum to represent the direction of travel in the context of
/// height.
#[derive(Clone, Debug)]
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
