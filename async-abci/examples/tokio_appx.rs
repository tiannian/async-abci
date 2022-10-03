use async_abci::ServerXX;
use tm_abci::{
    ConsensusXX, Mempool, Query, RequestFinalizedBlock, ResponseDeliverTx, ResponseFinalizedBlock,
    Snapshot,
};

#[derive(Debug, Clone)]
struct App {}

#[async_trait::async_trait]
impl ConsensusXX for App {
    async fn finalized_block(&self, req: RequestFinalizedBlock) -> ResponseFinalizedBlock {
        let mut fb = ResponseFinalizedBlock::default();

        for tx in req.transactions {
            let code = tx[0];

            let mut resp = ResponseDeliverTx::default();

            resp.code = code as u32;

            fb.tx_receipt.push(resp);
        }

        fb
    }
}

impl Query for App {}

impl Mempool for App {}

impl Snapshot for App {}

async fn start() {
    env_logger::init();

    let app = App {};

    ServerXX::new(app)
        .bind("127.0.0.1:26658")
        .await
        .unwrap()
        .run()
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    start().await;
}
