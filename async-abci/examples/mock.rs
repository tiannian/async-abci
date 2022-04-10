use async_abci::Server;
use std::{io, time::Duration};
use tm_abci::{Consensus, Mempool, Query, Snapshot};
use tm_protos::abci::{CheckTxType, RequestDeliverTx, ResponseDeliverTx};
use tokio::time::sleep;

#[derive(Debug, Clone)]
struct App {}

#[async_trait::async_trait]
impl Consensus for App {
    async fn deliver_tx(&self, _request: RequestDeliverTx) -> ResponseDeliverTx {
        sleep(Duration::from_secs(10)).await;

        Default::default()
    }
}

#[async_trait::async_trait]
impl Mempool for App {
    async fn check_tx(
        &self,
        _request: tm_protos::abci::RequestCheckTx,
    ) -> tm_protos::abci::ResponseCheckTx {
        if CheckTxType::from_i32(_request.r#type).unwrap() == CheckTxType::New {
            sleep(Duration::from_secs(2)).await;
        }

        Default::default()
    }
}

impl Snapshot for App {}

impl Query for App {}

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    let app = App {};

    Server::new(app)
        .bind("127.0.0.1:26658")
        .await
        .unwrap()
        .run()
        .await
        .unwrap();
    Ok(())
}
