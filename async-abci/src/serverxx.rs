use smol::{
    io::{AsyncRead, AsyncWrite},
    net::{unix::UnixListener, AsyncToSocketAddrs, TcpListener},
};
use tm_abci::ApplicationXX;

use crate::{
    codec::{ICodec, OCodec},
    state::{ConsensusQueue, State},
    Error, Result,
};

/// ACBI Server.
pub struct ServerXX<App> {
    #[cfg(feature = "tcp")]
    listener: Option<smol::net::TcpListener>,
    #[cfg(feature = "unix")]
    listener: Option<smol::net::unix::UnixListener>,
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

            smol::spawn(conn_handle(socket.clone(), socket, self.app.clone())).detach();
        }
    }
}

// async fn

async fn conn_handle<A, R, W>(reader: R, writer: W, app: A)
where
    R: AsyncRead + Unpin + Sync + Send + 'static,
    W: AsyncWrite + Unpin + Sync + Send + 'static,
    A: ApplicationXX + Clone + 'static,
{
    let mut state: Option<State> = None;
    let mut icodec = ICodec::new(reader, 4096);
    let mut ocodec = OCodec::new(writer);
    loop {
        let pkt = icodec.next().await;
        match pkt {
            Some(Ok(p)) => {
                if let Some(st) = &mut state {
                    // do logic of based on state.
                    if let State::Consensus(st) = st {
                        log::info!("State is: {:?}, packet is: {:?}", st.state, p);
                        st.add_pkt(p).expect("Error state convert");
                        if st.is_deliver_block() {
                            // do appxx
                            let fbp = st.to_block().expect("Failed to build block");
                            let res = app.finalized_block(fbp).await;
                            // app.dp).await;
                        }
                    } else {
                        // do appxx
                        let res = app.dispatch(p).await;
                        ocodec.send(res).await.expect("Failed to send data");
                    }
                } else {
                    // inital state
                    if ConsensusQueue::is_consensus(&p) {
                        state = Some(State::Consensus(
                            ConsensusQueue::new(p).expect("Logic error"),
                        ));
                    }
                }
            }
            Some(Err(e)) => {
                log::info!("Failed to read incoming request: {:?}", e);
            }
            None => {}
        }
    }
}
