//! Async abci application.
//!
//! Async version of abci.

use alloc::boxed::Box;

pub use tm_protos::abci::{
    request, response, Request, RequestApplySnapshotChunk, RequestBeginBlock, RequestCheckTx,
    RequestDeliverTx, RequestEcho, RequestEndBlock, RequestInfo, RequestInitChain,
    RequestLoadSnapshotChunk, RequestOfferSnapshot, RequestQuery, Response,
    ResponseApplySnapshotChunk, ResponseBeginBlock, ResponseCheckTx, ResponseCommit,
    ResponseDeliverTx, ResponseEcho, ResponseEndBlock, ResponseFlush, ResponseInfo,
    ResponseInitChain, ResponseListSnapshots, ResponseLoadSnapshotChunk, ResponseOfferSnapshot,
    ResponseQuery,
};

use crate::{RequestFinalizedBlock, ResponseFinalizedBlock};

/// Consensus trait, include `init_chain`, `begin_block`, `deliver_tx`, `end_block` and `commit`.
#[async_trait::async_trait]
pub trait Consensus {
    async fn init_chain(&self, _request: RequestInitChain) -> ResponseInitChain {
        Default::default()
    }

    async fn begin_block(&self, _request: RequestBeginBlock) -> ResponseBeginBlock {
        Default::default()
    }

    async fn deliver_tx(&self, _request: RequestDeliverTx) -> ResponseDeliverTx {
        Default::default()
    }

    async fn end_block(&self, _request: RequestEndBlock) -> ResponseEndBlock {
        Default::default()
    }

    async fn commit(&self) -> ResponseCommit {
        Default::default()
    }
}

#[async_trait::async_trait]
pub trait ConsensusXX {
    async fn init_chain(&self, _request: RequestInitChain) -> ResponseInitChain {
        Default::default()
    }

    async fn finalized_block(&self, _request: RequestFinalizedBlock) -> ResponseFinalizedBlock {
        Default::default()
    }

    async fn commit(&self) -> ResponseCommit {
        Default::default()
    }
}

/// Mempool, include `check_tx`.
#[async_trait::async_trait]
pub trait Mempool {
    async fn check_tx(&self, _request: RequestCheckTx) -> ResponseCheckTx {
        Default::default()
    }
}

/// Snapshot, include `list_snapshots`, `offer_snapshot`, `load_snapshot_chunk` and
/// `apply_snapshot_chunk`.
#[async_trait::async_trait]
pub trait Snapshot {
    async fn list_snapshots(&self) -> ResponseListSnapshots {
        Default::default()
    }

    async fn offer_snapshot(&self, _request: RequestOfferSnapshot) -> ResponseOfferSnapshot {
        Default::default()
    }

    async fn load_snapshot_chunk(
        &self,
        _request: RequestLoadSnapshotChunk,
    ) -> ResponseLoadSnapshotChunk {
        Default::default()
    }

    async fn apply_snapshot_chunk(
        &self,
        _request: RequestApplySnapshotChunk,
    ) -> ResponseApplySnapshotChunk {
        Default::default()
    }
}

/// Query, include `echo`, `info` and `query`.
#[async_trait::async_trait]
pub trait Query {
    async fn echo(&self, request: RequestEcho) -> ResponseEcho {
        ResponseEcho {
            message: request.message,
        }
    }

    async fn info(&self, _request: RequestInfo) -> ResponseInfo {
        Default::default()
    }

    async fn query(&self, _request: RequestQuery) -> ResponseQuery {
        Default::default()
    }
}

/// Async version application for ABCI.
#[async_trait::async_trait]
pub trait Application: Send + Sync + Consensus + Mempool + Snapshot + Query {
    async fn dispatch(&self, request: Request) -> Response {
        use request::Value;
        Response {
            value: Some(match request.value.unwrap() {
                Value::Echo(req) => response::Value::Echo(self.echo(req).await),
                Value::Flush(_) => response::Value::Flush(ResponseFlush {}),
                Value::Info(req) => response::Value::Info(self.info(req).await),
                Value::InitChain(req) => response::Value::InitChain(self.init_chain(req).await),
                Value::Query(req) => response::Value::Query(self.query(req).await),
                Value::BeginBlock(req) => response::Value::BeginBlock(self.begin_block(req).await),
                Value::CheckTx(req) => response::Value::CheckTx(self.check_tx(req).await),
                Value::DeliverTx(req) => response::Value::DeliverTx(self.deliver_tx(req).await),
                Value::EndBlock(req) => response::Value::EndBlock(self.end_block(req).await),
                Value::Commit(_) => response::Value::Commit(self.commit().await),
                Value::ListSnapshots(_) => {
                    response::Value::ListSnapshots(self.list_snapshots().await)
                }
                Value::OfferSnapshot(req) => {
                    response::Value::OfferSnapshot(self.offer_snapshot(req).await)
                }
                Value::LoadSnapshotChunk(req) => {
                    response::Value::LoadSnapshotChunk(self.load_snapshot_chunk(req).await)
                }
                Value::ApplySnapshotChunk(req) => {
                    response::Value::ApplySnapshotChunk(self.apply_snapshot_chunk(req).await)
                }
                Value::SetOption(_req) => response::Value::SetOption(Default::default()),
            }),
        }
    }
}

#[async_trait::async_trait]
pub trait ApplicationXX: Send + Sync + ConsensusXX + Mempool + Snapshot + Query {
    async fn dispatch(&self, request: Request) -> Response {
        use request::Value;
        Response {
            value: Some(match request.value.unwrap() {
                Value::Echo(req) => response::Value::Echo(self.echo(req).await),
                Value::Flush(_) => response::Value::Flush(ResponseFlush {}),
                Value::Info(req) => response::Value::Info(self.info(req).await),
                Value::InitChain(req) => response::Value::InitChain(self.init_chain(req).await),
                Value::Query(req) => response::Value::Query(self.query(req).await),
                // Note: This method will not call.
                Value::BeginBlock(_req) => response::Value::BeginBlock(Default::default()),
                Value::CheckTx(req) => response::Value::CheckTx(self.check_tx(req).await),
                // Note: This method will not call.
                Value::DeliverTx(_req) => response::Value::DeliverTx(Default::default()),
                // Note: This method will not call.
                Value::EndBlock(_req) => response::Value::EndBlock(Default::default()),
                Value::Commit(_) => response::Value::Commit(self.commit().await),
                Value::ListSnapshots(_) => {
                    response::Value::ListSnapshots(self.list_snapshots().await)
                }
                Value::OfferSnapshot(req) => {
                    response::Value::OfferSnapshot(self.offer_snapshot(req).await)
                }
                Value::LoadSnapshotChunk(req) => {
                    response::Value::LoadSnapshotChunk(self.load_snapshot_chunk(req).await)
                }
                Value::ApplySnapshotChunk(req) => {
                    response::Value::ApplySnapshotChunk(self.apply_snapshot_chunk(req).await)
                }
                Value::SetOption(_req) => response::Value::SetOption(Default::default()),
            }),
        }
    }
}

impl ConsensusXX for () {}

impl<T: ConsensusXX + Mempool + Snapshot + Query + Send + Sync> ApplicationXX for T {}

impl<T: Consensus + Mempool + Snapshot + Query + Send + Sync> Application for T {}

impl Consensus for () {}

impl Mempool for () {}

impl Snapshot for () {}

impl Query for () {}
