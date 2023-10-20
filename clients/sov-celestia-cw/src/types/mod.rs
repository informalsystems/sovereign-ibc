use alloc::vec::Vec;

pub mod client_message;
pub mod client_state;
pub mod consensus_state;
pub mod error;
pub mod proto;
pub mod serializer;

pub type Bytes = Vec<u8>;
