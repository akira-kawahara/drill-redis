//! DEL command
//! 
//! # command syntax
//! DEL key [key ...]
//! 
//! <https://redis.io/commands/del>
//! 
use crate::db;
use crate::protocol::resp::{Data, Parser};
use async_trait::async_trait;

/// Del commnad empty struct
pub(super) struct Del;

/// command register function
pub(super) fn command() -> (String, super::Cmd) {
    (String::from("DEL"), Box::new(Del))
}

#[async_trait]
impl super::Command for Del {
    /// Get command body
    async fn execute(&self, cmd: &mut Parser) -> crate::Result<Data> {
        let mut delete_num = 0;
        let mut key_exist = false;

        while let Some(key) = cmd.next_bytes()? {
            key_exist = true;

            if db::DB.write().await.del(key) {
                delete_num += 1;
            }
        }
        if key_exist {
            Ok(Data::Integer(delete_num))
        } else {
            Ok(Data::error("wrong number of arguments for command"))
        }
    }
}
