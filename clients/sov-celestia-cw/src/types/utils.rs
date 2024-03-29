use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;

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
