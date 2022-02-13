//! Async abci application.
//!
//! Async version of abci.

pub use tm_protos::abci::{
    request, response, Request, RequestApplySnapshotChunk, RequestBeginBlock, RequestCheckTx,
    RequestDeliverTx, RequestEcho, RequestEndBlock, RequestInfo, RequestInitChain,
    RequestLoadSnapshotChunk, RequestOfferSnapshot, RequestQuery, Response,
    ResponseApplySnapshotChunk, ResponseBeginBlock, ResponseCheckTx, ResponseCommit,
    ResponseDeliverTx, ResponseEcho, ResponseEndBlock, ResponseFlush, ResponseInfo,
    ResponseInitChain, ResponseListSnapshots, ResponseLoadSnapshotChunk, ResponseOfferSnapshot,
    ResponseQuery,
};

pub trait SyncApplication: Send {
    fn dispatch(&mut self, request: Request) -> Response {
        use request::Value;
        Response {
            value: Some(match request.value.unwrap() {
                Value::Echo(req) => response::Value::Echo(self.echo(req)),
                Value::Flush(_) => response::Value::Flush(self.flush()),
                Value::Info(req) => response::Value::Info(self.info(req)),
                Value::InitChain(req) => response::Value::InitChain(self.init_chain(req)),
                Value::Query(req) => response::Value::Query(self.query(req)),
                Value::BeginBlock(req) => response::Value::BeginBlock(self.begin_block(req)),
                Value::CheckTx(req) => response::Value::CheckTx(self.check_tx(req)),
                Value::DeliverTx(req) => response::Value::DeliverTx(self.deliver_tx(req)),
                Value::EndBlock(req) => response::Value::EndBlock(self.end_block(req)),
                Value::Commit(_) => response::Value::Commit(self.commit()),
                Value::ListSnapshots(_) => response::Value::ListSnapshots(self.list_snapshots()),
                Value::OfferSnapshot(req) => {
                    response::Value::OfferSnapshot(self.offer_snapshot(req))
                }
                Value::LoadSnapshotChunk(req) => {
                    response::Value::LoadSnapshotChunk(self.load_snapshot_chunk(req))
                }
                Value::ApplySnapshotChunk(req) => {
                    response::Value::ApplySnapshotChunk(self.apply_snapshot_chunk(req))
                }
            }),
        }
    }

    fn echo(&mut self, request: RequestEcho) -> ResponseEcho {
        ResponseEcho {
            message: request.message,
        }
    }

    fn info(&mut self, _request: RequestInfo) -> ResponseInfo {
        Default::default()
    }

    fn init_chain(&mut self, _request: RequestInitChain) -> ResponseInitChain {
        Default::default()
    }

    fn query(&mut self, _request: RequestQuery) -> ResponseQuery {
        Default::default()
    }

    fn check_tx(&mut self, _request: RequestCheckTx) -> ResponseCheckTx {
        Default::default()
    }

    fn begin_block(&mut self, _request: RequestBeginBlock) -> ResponseBeginBlock {
        Default::default()
    }

    fn deliver_tx(&mut self, _request: RequestDeliverTx) -> ResponseDeliverTx {
        Default::default()
    }

    fn end_block(&mut self, _request: RequestEndBlock) -> ResponseEndBlock {
        Default::default()
    }

    fn flush(&mut self) -> ResponseFlush {
        ResponseFlush {}
    }

    fn commit(&mut self) -> ResponseCommit {
        Default::default()
    }

    fn list_snapshots(&mut self) -> ResponseListSnapshots {
        Default::default()
    }

    fn offer_snapshot(&mut self, _request: RequestOfferSnapshot) -> ResponseOfferSnapshot {
        Default::default()
    }

    fn load_snapshot_chunk(
        &mut self,
        _request: RequestLoadSnapshotChunk,
    ) -> ResponseLoadSnapshotChunk {
        Default::default()
    }

    fn apply_snapshot_chunk(
        &mut self,
        _request: RequestApplySnapshotChunk,
    ) -> ResponseApplySnapshotChunk {
        Default::default()
    }
}

impl SyncApplication for () {}
