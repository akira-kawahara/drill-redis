//! Redis server.
//! 
use async_std::net::ToSocketAddrs;

mod handler;
mod listener;

/// Redis server main loop.
pub async fn run(addr: impl ToSocketAddrs) -> crate::Result<()> {
    let mut listener = listener::Listener::new(addr).await?;

    // Start the server.
    listener.listen().await?;

    // Server Terminated.
    Ok(())
}
