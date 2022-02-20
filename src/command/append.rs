//! APPEND command
//! 
//! # command syntax
//! APPEND key value
//! 
//! <https://redis.io/commands/append>
//! 
use crate::db;
use crate::protocol::resp::{Data, Parser};
use async_trait::async_trait;

/// Append commnad empty struct
pub(super) struct Append;

/// command register function
pub(super) fn command() -> (String, super::Cmd) {
    (String::from("APPEND"), Box::new(Append))
}

#[async_trait]
impl super::Command for Append {
    /// Get command body
    async fn execute(&self, cmd: &mut Parser) -> crate::Result<Data> {
        let key = super::next_bytes!(cmd);
        let value = super::next_bytes!(cmd);

        let length = db::DB.write().await.append(key, value);

        Ok(Data::Integer(length as i64))
    }
}
