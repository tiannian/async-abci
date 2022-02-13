#![no_std]
#![allow(rustdoc::bare_urls)]
#![allow(clippy::large_enum_variant)]

/// ABCI Message types.
pub mod abci {
    include!("protos/tendermint.abci.rs");
}

/// Types for ABCI.
pub mod types {
    include!("protos/tendermint.types.rs");
}

/// Version type.
pub mod version {
    include!("protos/tendermint.version.rs");
}

/// Crypto type.
pub mod crypto {
    include!("protos/tendermint.crypto.rs");
}
