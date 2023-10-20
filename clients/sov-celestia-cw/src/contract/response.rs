use alloc::string::String;
use alloc::vec::Vec;

use cosmwasm_schema::cw_serde;

use super::msgs::GenesisMetadata;

#[cw_serde]
pub struct QueryResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genesis_metadata: Option<Vec<GenesisMetadata>>,
}

impl QueryResponse {
    pub fn status(status: String) -> Self {
        Self {
            status,
            genesis_metadata: None,
        }
    }

    pub fn genesis_metadata(genesis_metadata: Option<Vec<GenesisMetadata>>) -> Self {
        Self {
            status: "".to_string(),
            genesis_metadata,
        }
    }
}

#[cw_serde]
pub struct ContractResult {
    pub is_valid: bool,
    pub error_msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<u8>>,
    pub found_misbehaviour: bool,
}

impl ContractResult {
    pub fn success() -> Self {
        Self {
            is_valid: true,
            error_msg: "".to_string(),
            data: None,
            found_misbehaviour: false,
        }
    }

    pub fn error(msg: String) -> Self {
        Self {
            is_valid: false,
            error_msg: msg,
            data: None,
            found_misbehaviour: false,
        }
    }

    pub fn misbehaviour(mut self, found: bool) -> Self {
        self.found_misbehaviour = found;
        self
    }

    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }
}
