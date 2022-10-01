#![no_std]
#![allow(rustdoc::bare_urls)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::derive_partial_eq_without_eq)]

/// ABCI Message types.
pub mod abci {
    include!(concat!(env!("OUT_DIR"), "/tendermint.abci.rs"));
}

/// Types for ABCI.
pub mod types {
    include!(concat!(env!("OUT_DIR"), "/tendermint.types.rs"));
}

/// Version type.
pub mod version {
    include!(concat!(env!("OUT_DIR"), "/tendermint.version.rs"));
}

/// Crypto type.
pub mod crypto {
    include!(concat!(env!("OUT_DIR"), "/tendermint.crypto.rs"));
}
