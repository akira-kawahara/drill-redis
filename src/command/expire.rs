//! EXPIRE, PEXPIRE command
//! 
//! # command syntax
//! EXPIRE key seconds [NX|XX|GT|LT]
//! 
//! <https://redis.io/commands/expire>
//! 
//! PEXPIRE key milliseconds [NX|XX|GT|LT]
//! 
//! <https://redis.io/commands/pexpire>
//! 
use crate::db;
use crate::protocol::resp::{Data, Parser};
use async_trait::async_trait;
use std::ops::Add;
use std::time::{Duration, Instant};

/// Expire commnad struct
pub(super) struct Expire {
    time_unit: super::TimeUnit,
}

/// command register function
pub(super) fn command(milliseconds: super::TimeUnit) -> (String, super::Cmd) {
    match milliseconds {
        super::TimeUnit::Second =>(String::from("PEXPIRE"), Box::new(Expire { time_unit })),
        super::TimeUnit::Millisecond =>(String::from("EXPIRE"), Box::new(Expire { time_unit }))   
    }
}

#[async_trait]
impl super::Command for Expire {
    /// Get command body
    async fn execute(&self, cmd: &mut Parser) -> crate::Result<Data> {
        let key = super::next_bytes!(cmd);
        let expiration = match self.time_unit {
            super::TimeUnit::Second =>Some(Instant::now().add(Duration::from_millis(super::next_u64!(cmd)))),
            super::TimeUnit::Millisecond =>Some(Instant::now().add(Duration::from_secs(super::next_u64!(cmd)))),   
        };

        let set_condition;

        match cmd.next_string()? {
            Some(param) => {
                match param.as_str() {
                    "NX" => set_condition = db::SetCondition::NX,
                    "XX" => set_condition = db::SetCondition::XX,
                    "GT" => set_condition = db::SetCondition::GT,
                    "LT" => set_condition = db::SetCondition::LT,
                    _ => {
                        return Ok(Data::error("syntax error"));
                    }
                }
                super::check_end_of_param!(cmd);
            }
            None => set_condition = db::SetCondition::NONE,
        }
        if db::DB.write().await.expire(key, expiration, set_condition) {
            Ok(Data::Integer(1))
        } else {
            Ok(Data::Integer(0))
        }
    }
}
