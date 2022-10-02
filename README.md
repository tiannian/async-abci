# async-abci

An ABCI like [tendermint-abci](https://crates.io/crates/tendermint-abci), but asynchronous.

## Version

- tendermint: 0.35.x
- spec: 0.7.1

## Packages

| name | description | crates.io | docs.rs |
| - | - | - | - |
| async-abci | use tendermint in Rust | ![Crates.io](https://img.shields.io/crates/v/async-abci) | ![docs.rs](https://img.shields.io/docsrs/async-abci) |
| tm-abci | ABCI interface in `no_std` | ![Crates.io](https://img.shields.io/crates/v/tm-abci) | ![docs.rs](https://img.shields.io/docsrs/tm-abci) |
| tm-protos | ABCI types in `no_std` | ![Crates.io](https://img.shields.io/crates/v/tm-protos) | ![docs.rs](https://img.shields.io/docsrs/tm-protos) |

## Features

- async-abci: async version of ABCI, fully cooperate with `Flush` in ABCI message.
- Async runtime support: tokio.

## Design

### Consensus

![state machine](assets/consensus.png)

