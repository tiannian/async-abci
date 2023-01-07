use smol::{
    io::{AsyncRead, AsyncWrite},
    net::{AsyncToSocketAddrs, TcpListener},
};

use tm_abci::{request, response, ApplicationXX, Request, Response};

use crate::{
    codec::{ICodec, OCodec},
    state::ConsensusQueue,
    Error, Result,
};

/// ACBI Server.
pub struct ServerXX<App> {
    tcp: Option<TcpListener>,
    #[cfg(unix)]
    unix: Option<smol::net::unix::UnixListener>,
    app: App,
}

impl<App> ServerXX<App>
where
    App: ApplicationXX + Clone + 'static,
{
    pub fn new(app: App) -> Self {
        Self {
            tcp: None,

            #[cfg(unix)]
            unix: None,

            app,
        }
    }

    pub async fn bind<A: AsyncToSocketAddrs>(mut self, addr: A) -> Result<Self> {
        let tcp = TcpListener::bind(addr).await?;
        self.tcp = Some(tcp);

        Ok(self)
    }

    #[cfg(unix)]
    pub async fn bind_unix<P: AsRef<std::path::Path>>(mut self, path: P) -> Result<Self> {
        let unix = smol::net::unix::UnixListener::bind(path)?;
        self.unix = Some(unix);

        Ok(self)
    }

    pub async fn run(self) -> Result<()> {
        let _unix_exist = false;

        #[cfg(unix)]
        let _unix_exist = self.unix.is_none();

        if self.tcp.is_none() && _unix_exist {
            return Err(Error::ServerNotBinding);
        }

        if let Some(tcp) = self.tcp {
            let (socket, addr) = tcp.accept().await?;
            log::info!("new connect from {:?}", addr);

            smol::spawn(conn_handle(socket.clone(), socket, self.app.clone())).detach();
        }

        #[cfg(unix)]
        if let Some(unix) = self.unix {
            let (socket, addr) = unix.accept().await?;
            log::info!("new connect from {:?}", addr);

            smol::spawn(conn_handle(socket.clone(), socket, self.app.clone())).detach();
        }

        Ok(())
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

                    if st.flushed {
                        send_flush(&mut ocodec).await;
                        continue;
                    }

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
