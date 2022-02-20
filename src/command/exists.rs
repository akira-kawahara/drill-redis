//! EXISTS command
//! 
//! # command syntax
//! EXISTS key [key ...]
//! 
//! <https://redis.io/commands/exists>
//! 
use crate::db;
use crate::protocol::resp::{Data, Parser};
use async_trait::async_trait;

/// Exists commnad empty struct
pub(super) struct Exists;

/// command register function
pub(super) fn command() -> (String, super::Cmd) {
    (String::from("EXISTS"), Box::new(Exists))
}

#[async_trait]
impl super::Command for Exists {
    /// Get command body
    async fn execute(&self, cmd: &mut Parser) -> crate::Result<Data> {
        let mut exist_num = 0;
        let mut key_exist = false;

        while let Some(key) = cmd.next_bytes()? {
            key_exist = true;
            if let Some(_) = db::DB.write().await.get(&key) {
                exist_num += 1;
            }
        }
        if key_exist {
            Ok(Data::Integer(exist_num))
        } else {
            Ok(Data::error("wrong number of arguments for command"))
        }
    }
}
