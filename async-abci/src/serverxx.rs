use smol::net::{AsyncToSocketAddrs, TcpListener};
use tm_abci::ApplicationXX;

use crate::Result;

/// ACBI Server.
pub struct Server<App> {
    #[cfg(feature = "tcp")]
    tcp_listener: Option<smol::net::TcpListener>,
    #[cfg(feature = "unix")]
    unix_listener: Option<smol::net::unix::UnixListener>,
    app: App,
}

impl<App> Server<App>
where
    App: ApplicationXX + Clone + 'static,
{
    pub fn new(app: App) -> Self {
        Self {
            #[cfg(feature = "tcp")]
            tcp_listener: None,

            #[cfg(feature = "unix")]
            unix_listener: None,

            app,
        }
    }

    // #[cfg(feature = "tcp")]
    pub async fn bind<A: AsyncToSocketAddrs>(mut self, addr: A) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        self.tcp_listener = Some(listener);
        Ok(self)
    }
}
