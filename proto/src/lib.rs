//! sov-ibc-proto library gives the developer access to the Sovereign SDK IBC proto-defined structs.
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate core as std;

/// File descriptor set of compiled proto.
#[cfg(feature = "proto-descriptor")]
pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("prost/proto_descriptor.bin");

#[macro_export]
macro_rules! include_proto {
    ($path:literal) => {
        include!(concat!("prost/", $path));
    };
}

pub mod ibc {
    pub mod lightclients {
        pub mod sovereign {
            pub mod tendermint {
                pub mod v1 {
                    include_proto!("ibc.lightclients.sovereign.tendermint.v1.rs");
                    #[cfg(feature = "serde")]
                    include_proto!("ibc.lightclients.sovereign.tendermint.v1.serde.rs");
                }
            }
        }
    }
}

pub mod sovereign {
    pub mod types {
        pub mod v1 {
            include_proto!("sovereign.types.v1.rs");
            #[cfg(feature = "serde")]
            include_proto!("sovereign.types.v1.serde.rs");
        }
    }
}
