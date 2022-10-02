#[cfg(feature = "smol-backend")]
use smol::io::{AsyncRead, AsyncWrite};
#[cfg(all(feature = "smol-backend", feature = "unix"))]
use smol::net::unix::UnixListener;
#[cfg(all(feature = "smol-backend", feature = "tcp"))]
use smol::net::{AsyncToSocketAddrs, TcpListener};

#[cfg(feature = "tokio-backend")]
use tokio::io::{AsyncRead, AsyncWrite};
#[cfg(all(feature = "tokio-backend", feature = "unix"))]
use tokio::net::UnixListener;
#[cfg(all(feature = "tokio-backend", feature = "tcp"))]
use tokio::net::{TcpListener, ToSocketAddrs as AsyncToSocketAddrs};

use tm_abci::{request, response, ApplicationXX, Request, Response};

use crate::{
    codec::{ICodec, OCodec},
    state::ConsensusQueue,
    Error, Result,
};

/// ACBI Server.
pub struct ServerXX<App> {
    #[cfg(feature = "tcp")]
    listener: Option<TcpListener>,
    #[cfg(feature = "unix")]
    listener: Option<UnixListener>,
    app: App,
}

impl<App> ServerXX<App>
where
    App: ApplicationXX + Clone + 'static,
{
    pub fn new(app: App) -> Self {
        Self {
            #[cfg(feature = "tcp")]
            listener: None,

            #[cfg(feature = "unix")]
            listener: None,

            app,
        }
    }

    #[cfg(feature = "tcp")]
    pub async fn bind<A: AsyncToSocketAddrs>(mut self, addr: A) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        self.listener = Some(listener);

        Ok(self)
    }

    #[cfg(feature = "unix")]
    pub async fn bind_unix<P: AsRef<std::path::Path>>(mut self, path: P) -> Result<Self> {
        let listener = UnixListener::bind(path)?;
        self.listener = Some(listener);

        Ok(self)
    }

    pub async fn run(self) -> Result<()> {
        if self.listener.is_none() {
            return Err(Error::ServerNotBinding);
        }
        let listener = self.listener.unwrap();
        loop {
            let (socket, addr) = listener.accept().await?;
            log::info!("new connect from {:?}", addr);

            #[cfg(feature = "smol-backend")]
            smol::spawn(conn_handle(socket.clone(), socket, self.app.clone())).detach();

            #[cfg(feature = "tokio-backend")]
            {
                let (reader, writer) = socket.into_split();
                tokio::spawn(conn_handle(reader, writer, self.app.clone()));
            }
        }
    }
}

async fn send_flush<W>(ocodec: &mut OCodec<W>)
where
    W: AsyncWrite + Unpin + Sync + Send + 'static,
{
    let flush = Response {
        value: Some(response::Value::Flush(Default::default())),
    };
    log::info!("Send: {:?}", flush);
    ocodec.send(flush).await.expect("Failed to send data");
}

async fn send_response<W>(ocodec: &mut OCodec<W>, resp: Response)
where
    W: AsyncWrite + Unpin + Sync + Send + 'static,
{
    log::info!("Send: {:?}", resp);
    ocodec.send(resp).await.expect("Failed to send data");
}

async fn conn_handle<A, R, W>(reader: R, writer: W, app: A)
where
    R: AsyncRead + Unpin + Sync + Send + 'static,
    W: AsyncWrite + Unpin + Sync + Send + 'static,
    A: ApplicationXX + Clone + 'static,
{
    let mut state: Option<ConsensusQueue> = None;
    let mut icodec = ICodec::new(reader, 4096);
    let mut ocodec = OCodec::new(writer);
    loop {
        let pkt = icodec.next().await;
        log::info!("Recv: {:?}", pkt);

        match pkt {
            Some(Ok(p)) => {
                log::info!("Recv: {:?}", p);

                if state.is_none() && ConsensusQueue::is_consensus(&p) {
                    state = Some(ConsensusQueue::new(p.clone()).expect("Logic error"));
                }

                if let Some(st) = &mut state {
                    // do logic of based on state.
                    log::info!("State is: {:?}", st.state);
                    st.add_pkt(p).expect("Error state convert");
                    if st.is_deliver_block() {
                        // do appxx
                        let fbp = st.to_block().expect("Failed to build block");

                        let tx_len = fbp.transactions.len();

                        let res = app.finalized_block(fbp).await;

                        let filled_tx = tx_len - res.tx_receipt.len();

                        for tx in res.tx_receipt {
                            let value = Some(response::Value::DeliverTx(tx));
                            let resp = Response { value };

                            send_response(&mut ocodec, resp).await;
                        }

                        for _ in 0..filled_tx {
                            let value = Some(response::Value::DeliverTx(Default::default()));
                            let resp = Response { value };

                            send_response(&mut ocodec, resp).await;
                        }

                        let value = Some(response::Value::EndBlock(res.end_recepit));
                        let resp = Response { value };

                        send_response(&mut ocodec, resp).await;

                        send_flush(&mut ocodec).await;
                    }

                    if st.is_sendable() {
                        let request = st.to_packet().expect("Wrong state");
                        let resp = app.dispatch(request).await;

                        send_response(&mut ocodec, resp).await;

                        send_flush(&mut ocodec).await;
                    }

                    if st.is_begin_block_flush() {
                        let resp = Response {
                            value: Some(response::Value::BeginBlock(Default::default())),
                        };

                        send_response(&mut ocodec, resp).await;

                        send_flush(&mut ocodec).await;
                    }

                    if st.is_commit_flush() {
                        let req = Request {
                            value: Some(request::Value::Commit(Default::default())),
                        };
                        let resp = app.dispatch(req).await;

                        send_response(&mut ocodec, resp).await;

                        send_flush(&mut ocodec).await;
                    }
                } else {
                    // do appxx
                    let res = app.dispatch(p).await;
                    log::info!("Send: {:?}", res);
                    ocodec.send(res).await.expect("Failed to send data");
                }
            }
            Some(Err(e)) => {
                log::info!("Failed to read incoming request: {:?}", e);
            }
            None => {}
        }
    }
}
