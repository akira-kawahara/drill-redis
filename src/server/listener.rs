//! Listener 
//! 
//! Listen to the client's connection request. 
use super::handler;
use crate::db;
use async_std::{
    channel,
    net::{TcpListener, ToSocketAddrs},
    prelude::*,
    task,
};
use futures::{select, FutureExt};
use signal_hook::consts::signal::*;
use signal_hook_async_std::Signals;

/// Listener
pub(crate) struct Listener {
    tcp_listener: TcpListener,
}

impl Listener {
    /// create Listener instance.    
    pub(crate) async fn new(addr: impl ToSocketAddrs) -> crate::Result<Self> {
        let tcp_listener = TcpListener::bind(addr).await?;
        Ok(Listener { tcp_listener })
    }
    /// Listen to the client's connection request. 
    pub(crate) async fn listen(&mut self) -> crate::Result<()> {
        let (shutdown_tx, shutdown_rx) = channel::bounded::<crate::Void>(1);
        let (shutdown_complete_tx, mut shutdown_complete_rx) = channel::bounded::<crate::Void>(1);

        //Open database.
        db::open(shutdown_rx.clone()).await;

        //Signals to handle
        let mut signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;

        loop {
            let mut incoming = self.tcp_listener.incoming();

            select! {
                // Wait for incoming.
                stream = incoming.next().fuse() => match stream {
                    Some(stream) => {
                        let stream = stream?;
                        // The channel for shutdown completion notification.
                        let shutdown_complete = shutdown_complete_tx.clone();
                        // The broadcast channel for shutdown notification.
                        let shutdown = shutdown_rx.clone();
                        task::spawn(async move {
                            let mut handler = handler::Handler::new(stream, shutdown_complete);
                            if let Err(e) = handler.run(shutdown).await {
                                eprintln!("{}", e);
                            }
                        });
                    },
                    None => break,
                },
                // Wait for signals.
                _ = signals.next().fuse() => {
                    break;
                },
            }
        }

        // Notify all tasks of shutdown.
        drop(shutdown_tx);

        // Drop the channel that are not in use.
        drop(shutdown_complete_tx);

        // Wait for all tasks to end.
        shutdown_complete_rx.next().await;

        // Stop database
        db::close().await;

        Ok(())
    }
}
