//! GET command
//! 
//! # command syntax
//! GET key
//! 
//! <https://redis.io/commands/get>
//! 
use crate::db;
use crate::protocol::resp::{Data, Parser};
use async_trait::async_trait;

/// Get commnad empty struct
pub(super) struct Get;

/// command register function
pub(super) fn command() -> (String, super::Cmd) {
    (String::from("GET"), Box::new(Get))
}

#[async_trait]
impl super::Command for Get {
    /// Get command body
    async fn execute(&self, cmd: &mut Parser) -> crate::Result<Data> {
        let key = super::next_bytes!(cmd);
        super::check_end_of_param!(cmd);

        match db::DB.read().await.get_value(&key) {
            Some(value) => Ok(Data::checked_bulk(value)),
            None => Ok(Data::NullBulk),
        }
    }
}
