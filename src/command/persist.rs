//! PERSIST command
//! 
//! # command syntax
//! PERSIST key
//! 
//! <https://redis.io/commands/persist>
//! 
use crate::db;
use crate::protocol::resp::{Data, Parser};
use async_trait::async_trait;

/// Persist commnad empty struct
pub(super) struct Persist;

/// command register function
pub(super) fn command() -> (String, super::Cmd) {
    (String::from("PERSIST"), Box::new(Persist))
}

#[async_trait]
impl super::Command for Persist {
    /// Get command body     
    async fn execute(&self, cmd: &mut Parser) -> crate::Result<Data> {
        let key = super::next_bytes!(cmd);
        super::check_end_of_param!(cmd);

        if db::DB.write().await.persist(key) {
            return Ok(Data::Integer(1));
        } else {
            return Ok(Data::Integer(0));
        }
    }
}
