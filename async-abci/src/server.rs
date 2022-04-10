use crate::codec::OCodec;
use crate::{codec::ICodec, Error, Result};
use std::collections::BTreeMap;
use tm_abci::Application;
use tm_protos::abci::{request, response, Response, ResponseFlush};
use tokio::io::AsyncWrite;
use tokio::{
    io::AsyncRead,
    sync::mpsc::{unbounded_channel, UnboundedSender},
};

pub const DEFAULT_SERVER_READ_BUF_SIZE: usize = 1024 * 1024;

fn is_flush_reponse(resp: &Response) -> bool {
    match resp.value {
        Some(response::Value::Flush(_)) => true,
        _ => false,
    }
}

fn build_flush_resp() -> Response {
    Response {
        value: Some(response::Value::Flush(ResponseFlush {})),
    }
}

#[allow(unused_assignments)]
async fn read_to_flush<I: AsyncRead + Unpin, A: Application + Clone + 'static>(
    codec: &mut ICodec<I>,
    app: A,
    resp_tx: UnboundedSender<(usize, Response)>,
) -> Option<usize> {
    // Read packet to flush, return count of non-empty packet.
    let mut packet_num = 0;
    let mut end_block = None;

    loop {
        let app = app.clone();
        let resp_tx = resp_tx.clone();

        match codec.next().await {
            Some(Ok(req)) => {
                match req.value {
                    Some(request::Value::EndBlock(_)) => {
                        // Store EndBlock, Wait flush.
                        end_block = Some(req);
                    }
                    Some(request::Value::Flush(_)) => {
                        // Flush this window.
                        if let Some(r) = end_block.clone() {
                            packet_num += 1;

                            // Call end block direct.
                            log::debug!("Window id: {} Recv request: {:?}", packet_num, r);
                            let resp = app.dispatch(r.clone()).await;
                            resp_tx.send((packet_num, resp)).unwrap();
                        }

                        packet_num += 1;

                        log::debug!("Window id: {} Recv request: {:?}", packet_num, req);
                        resp_tx.send((packet_num, build_flush_resp())).unwrap();

                        let pn = packet_num;

                        packet_num = 0;

                        return Some(pn);
                    }
                    _ => {
                        packet_num += 1;

                        log::debug!("Window id: {} Recv request: {:?}", packet_num, req);
                        tokio::spawn(async move {
                            let resp = app.dispatch(req.clone()).await;
                            resp_tx.send((packet_num, resp)).unwrap();
                        });
                    }
                }
            }
            Some(Err(e)) => {
                log::info!("Failed to read incoming request: {:?}", e);
                return None;
            }
            None => return None,
        }
    }
}

#[allow(unused_assignments)]
async fn conn_handle<A, R, W>(reader: R, writer: W, app: A)
where
    R: AsyncRead + Unpin + Sync + Send + 'static,
    W: AsyncWrite + Unpin + Sync + Send + 'static,
    A: Application + Clone + 'static,
{
    let mut icodec = ICodec::new(reader, DEFAULT_SERVER_READ_BUF_SIZE);
    let mut ocodec = OCodec::new(writer);

    let (resp_tx, mut resp_rx) = unbounded_channel::<(usize, Response)>();

    tokio::spawn(async move {
        let mut resps = BTreeMap::new();
        let mut lastest_packet_number = 0;
        let mut first_packet_number = 0;

        loop {
            if let Some(resp) = resp_rx.recv().await {
                // insert resps.
                resps.insert(resp.0, resp.1);
                first_packet_number = first_index(&resps);

                // process resps.
                loop {
                    log::debug!(
                        "Will send packet: {}, expect: {}",
                        first_packet_number,
                        lastest_packet_number + 1
                    );
                    if first_packet_number == lastest_packet_number + 1 {
                        if let Some(v) = resps.remove(&first_packet_number) {
                            // Send v.
                            if is_flush_reponse(&v) {
                                resps.clear();
                                lastest_packet_number = 0;
                                first_packet_number = 0;
                            }

                            log::debug!("Window id: {}, packet sent: {:?}", first_packet_number, v);
                            ocodec.send(v).await.unwrap();
                            lastest_packet_number = first_packet_number;
                            first_packet_number = first_index(&resps);
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    });

    loop {
        let app = app.clone();
        let resp_tx = resp_tx.clone();

        if let Some(expect_packet_num) = read_to_flush(&mut icodec, app, resp_tx.clone()).await {
            log::debug!("Recv {} packet before flush.", expect_packet_num);
        } else {
            return;
        }
    }
}

fn first_index(resps: &BTreeMap<usize, Response>) -> usize {
    if let Some((k, _)) = resps.iter().next() {
        *k
    } else {
        0
    }
}

/// ACBI Server.
pub struct Server<A> {
    #[cfg(feature = "tcp")]
    listener: Option<tokio::net::TcpListener>,
    #[cfg(feature = "unix")]
    listener: Option<tokio::net::UnixListener>,
    app: A,
}

impl<A: Application + Clone + 'static> Server<A> {
    pub fn new(app: A) -> Self {
        Server {
            listener: None,
            app,
        }
    }

    #[cfg(feature = "tcp")]
    pub async fn bind<Addr: tokio::net::ToSocketAddrs>(mut self, addr: Addr) -> Result<Self> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        self.listener = Some(listener);
        Ok(self)
    }

    #[cfg(feature = "unix")]
    pub async fn bind_unix<P: AsRef<std::path::Path>>(mut self, path: P) -> Result<Self> {
        let listener = tokio::net::UnixListener::bind(path)?;
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

            let (reader, writer) = socket.into_split();
            tokio::spawn(conn_handle(reader, writer, self.app.clone()));
        }
    }
}
