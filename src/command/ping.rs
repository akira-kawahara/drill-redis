//! PING command
//! 
//! # command syntax
//! PING \[message\]
//! 
//! <https://redis.io/commands/ping>
//! 
use crate::protocol::resp::{Data, Parser};
use async_trait::async_trait;

/// Ping commnad empty struct
pub(super) struct Ping;

/// command register function
pub(super) fn command() -> (String, super::Cmd) {
    (String::from("PING"), Box::new(Ping))
}

#[async_trait]
impl super::Command for Ping {
    /// Get command body       
    async fn execute(&self, cmd: &mut Parser) -> crate::Result<Data> {
        match cmd.next_bytes()? {
            Some(echo) => {
                super::check_end_of_param!(cmd);

                Ok(Data::Bulk(echo))
            }
            None => Ok(Data::pong()),
        }
    }
}
