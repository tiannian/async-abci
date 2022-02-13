#![no_std]
#![allow(rustdoc::bare_urls)]

pub mod abci {
    include!("protos/tendermint.abci.rs");
}

pub mod types {
    include!("protos/tendermint.types.rs");
}

pub mod version {
    include!("protos/tendermint.version.rs");
}

pub mod crypto {
    include!("protos/tendermint.crypto.rs");
}
