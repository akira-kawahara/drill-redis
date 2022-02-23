//! SET command
//! 
//! # command syntax
//! SET key value [EX seconds|PX milliseconds|KEEPTTL] [NX|XX] \[GET\]
//! 
//! <https://redis.io/commands/set>
//! 
use crate::db;
use crate::protocol::resp::{Data, Parser};
use async_trait::async_trait;
use std::ops::Add;
use std::time::{Duration, Instant};

/// Set commnad empty struct
pub(super) struct Set;

/// command register function
pub(super) fn command() -> (String, super::Cmd) {
    (String::from("SET"), Box::new(Set))
}

#[async_trait]
impl super::Command for Set {
    /// Get command body       
    async fn execute(&self, cmd: &mut Parser) -> crate::Result<Data> {
        let key = super::next_bytes!(cmd);
        let value = super::next_bytes!(cmd);

        let mut expiration: Option<Instant> = None;
        let mut set_condition = db::SetCondition::NONE;
        let mut get = false;
        let mut keep_ttl = false;

        while let Some(param) = cmd.next_string()? {
            match param.as_str() {
                "EX" => match expiration {
                    Some(_) => {
                        return Ok(Data::error("syntax error"));
                    }
                    None => {
                        if keep_ttl {
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
                        if keep_ttl {
                            return Ok(Data::error("syntax error"));
                        }
                        let duration = super::next_u64!(cmd);
                        expiration = Some(Instant::now().add(Duration::from_millis(duration)));
                    }
                },
                "KEEPTTL" => match expiration {
                    Some(_) => {
                        return Ok(Data::error("syntax error"));
                    }
                    None => {
                        if keep_ttl {
                            return Ok(Data::error("syntax error"));
                        }
                        keep_ttl = true;
                    }
                },
                "NX" => match set_condition {
                    db::SetCondition::NONE => set_condition = db::SetCondition::NX,
                    _ => {
                        return Ok(Data::error("syntax error"));
                    }
                },
                "XX" => match set_condition {
                    db::SetCondition::NONE => set_condition = db::SetCondition::XX,
                    _ => {
                        return Ok(Data::error("syntax error"));
                    }
                },
                "GET" => {
                    if get {
                        return Ok(Data::error("syntax error"));
                    }
                    get = true;
                }
                _ => {
                    return Ok(Data::error("syntax error"));
                }
            }
        }

        match db::DB
            .write()
            .await
            .set(key, value, expiration, set_condition, keep_ttl, get)
        {
            Some(value) => Ok(Data::Bulk(value)),
            None => {
                if get {
                    Ok(Data::NullBulk)
                } else {
                    Ok(Data::ok())
                }
            }
        }
    }
}
