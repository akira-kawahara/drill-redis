//! Handler
//! 
//! Handle client requests.
use crate::{
    command,
    protocol,
    protocol::resp::Decoder,
};
use async_std::{
    io::BufReader,
    io::BufWriter,
    channel,
    net::TcpStream, 
    prelude::*};
use futures::{select, FutureExt};
use std::net::Shutdown;

// Handler
pub(crate) struct Handler {
    /// TCP/IP stream
    stream: TcpStream,
    /// The channel for shutdown completion notification.
    /// The Listener can recognize shutdown completion by being dropped.
    _shutdown_complete: channel::Sender<crate::Void>,
}

impl Handler {
    /// create Handler instance.
    pub(crate) fn new(stream: TcpStream, shutdown_complete: channel::Sender<crate::Void>) -> Self {
        Handler {
            stream,
            _shutdown_complete: shutdown_complete,
        }
    }
    /// Handle client requests.
    pub(crate) async fn run(
        &mut self,
        mut shutdown_event: channel::Receiver<crate::Void>,
    ) -> crate::Result<()> {
        let mut decoder = Decoder::new();
        let mut writer = BufWriter::new(&self.stream);
        let mut reader = BufReader::new(&self.stream);
        loop {
            let data = select! {
                // Read bytes from the stream and deocde it.
                // If the client unilaterally disconnects, it will remain connected.
                ret = decoder.decode(&mut reader).fuse() => match ret {
                    Ok(data) => data,
                    Err(e) => {
                        match e.downcast_ref::<protocol::Error>() {
                            Some(decode_err) => {
                                match decode_err {
                                    protocol::Error::ProtcolError => {
                                        self.close();
                                        return Err(e);
                                    },
                                    protocol::Error::ConnectionClosed => {
                                        return Ok(());
                                    },
                                }
                            },
                            _ =>{
                                self.close();
                                return Err(e);
                            },
                        }
                    },
                },
                // Wait for a shutdown.                
                void = shutdown_event.next().fuse() => match void {
                    Some(void) => match void {},
                    None => {
                        self.close();
                        return Ok(());
                    },
                }
            };
            //　Execute requested command.
            command::execute(&mut writer, data).await?;
        }
    }
    /// Close handler.
    pub(crate) fn close(&mut self) {
        match self.stream.shutdown(Shutdown::Both) {
            Err(_e) => {}
            Ok(()) => {}
        }
    }
}
