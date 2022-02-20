//! GETEX command
//! 
//! # command syntax
//! GETEX key [EX seconds|PX milliseconds|PERSIST]
//! 
//! <https://redis.io/commands/getex>
//! 
use crate::db;
use crate::protocol::resp::{Data, Parser};
use async_trait::async_trait;
use std::ops::Add;
use std::time::{Duration, Instant};

/// GetEx commnad empty struct
pub(super) struct GetEx;

/// command register function
pub(super) fn command() -> (String, super::Cmd) {
    (String::from("GETEX"), Box::new(GetEx))
}

#[async_trait]
impl super::Command for GetEx {
    /// Get command body    
    async fn execute(&self, cmd: &mut Parser) -> crate::Result<Data> {
        let key = super::next_bytes!(cmd);

        let mut expiration: Option<Instant> = None;
        let mut persist = false;

        while let Some(param) = cmd.next_string()? {
            match param.as_str() {
                "EX" => match expiration {
                    Some(_) => {
                        return Ok(Data::error("syntax error"));
                    }
                    None => {
                        if persist {
                            return Ok(Data::error("syntax error"));
                        }
                        let duration = super::next_u64!(cmd);
                        expiration = Some(Instant::now().add(Duration::from_secs(duration)));
                    }
                },
                "PX" => match expiration {
                    Some(_) => {
                        return Ok(Data::error("syntax error"));
                    }
                    None => {
                        if persist {
                            return Ok(Data::error("syntax error"));
                        }
                        let duration = super::next_u64!(cmd);
                        expiration = Some(Instant::now().add(Duration::from_millis(duration)));
                    }
                },
                "PERSIST" => match expiration {
                    Some(_) => {
                        return Ok(Data::error("syntax error"));
                    }
                    None => {
                        if persist {
                            return Ok(Data::error("syntax error"));
                        }
                        persist = true;
                    }
                },
                _ => {
                    return Ok(Data::error("syntax error"));
                }
            }
        }

        match db::DB.write().await.getex(key, expiration, persist) {
            Some(value) => Ok(Data::Bulk(value)),
            None => Ok(Data::NullBulk),
        }
    }
}
