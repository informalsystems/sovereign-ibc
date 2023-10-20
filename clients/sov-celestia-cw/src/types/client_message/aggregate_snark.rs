use alloc::vec::Vec;

use crate::types::error::Error;
use crate::types::proto::AggregateSNARK as RawAggregateSNARK;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AggregateSNARK {
    pub start_da_hash: Vec<u8>,
    pub end_da_hash: Vec<u8>,
    pub input_state_root: Vec<u8>,
    pub output_state_root: Vec<u8>,
}

impl TryFrom<RawAggregateSNARK> for AggregateSNARK {
    type Error = Error;

    fn try_from(raw: RawAggregateSNARK) -> Result<Self, Self::Error> {
        Ok(Self {
            start_da_hash: raw.start_da_hash.ok_or(Error::missing("start_da_hash"))?,
            end_da_hash: raw.end_da_hash.ok_or(Error::missing("end_da_hash"))?,
            input_state_root: raw
                .input_state_root
                .ok_or(Error::missing("input_state_root"))?,
            output_state_root: raw
                .output_state_root
                .ok_or(Error::missing("output_state_root"))?,
        })
    }
}

impl From<AggregateSNARK> for RawAggregateSNARK {
    fn from(snark: AggregateSNARK) -> Self {
        Self {
            start_da_hash: Some(snark.start_da_hash),
            end_da_hash: Some(snark.end_da_hash),
            input_state_root: Some(snark.input_state_root),
            output_state_root: Some(snark.output_state_root),
        }
    }
}
