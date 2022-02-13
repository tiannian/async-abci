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

/// Async version application for ABCI.
#[async_trait::async_trait]
pub trait Application: Send + Sync {
    async fn dispatch(&self, request: Request) -> Response {
        use request::Value;
        Response {
            value: Some(match request.value.unwrap() {
                Value::Echo(req) => response::Value::Echo(self.echo(req).await),
                Value::Flush(_) => response::Value::Flush(self.flush().await),
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
            }),
        }
    }

    async fn echo(&self, request: RequestEcho) -> ResponseEcho {
        ResponseEcho {
            message: request.message,
        }
    }

    async fn info(&self, _request: RequestInfo) -> ResponseInfo {
        Default::default()
    }

    async fn init_chain(&self, _request: RequestInitChain) -> ResponseInitChain {
        Default::default()
    }

    async fn query(&self, _request: RequestQuery) -> ResponseQuery {
        Default::default()
    }

    async fn check_tx(&self, _request: RequestCheckTx) -> ResponseCheckTx {
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

    async fn flush(&self) -> ResponseFlush {
        ResponseFlush {}
    }

    async fn commit(&self) -> ResponseCommit {
        Default::default()
    }

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

impl Application for () {}
