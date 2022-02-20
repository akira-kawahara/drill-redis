//! TTL, PTTL command
//! 
//! # command syntax
//! TTL key
//! 
//! <https://redis.io/commands/ttl>
//! 
//! PTTL key
//! 
//! <https://redis.io/commands/pttl>
//! 
use crate::db;
use crate::protocol::resp::{Data, Parser};
use async_trait::async_trait;
use std::time::Instant;

/// TTL commnad struct
pub(crate) struct TTL {
    milliseconds: super::TimeUnit,
}

/// command register function
pub(crate) fn command(milliseconds: super::TimeUnit) -> (String, super::Cmd) {
    match milliseconds {
        super::TimeUnit::Second =>(String::from("PTTL"), Box::new(TTL { milliseconds })),
        super::TimeUnit::Millisecond =>(String::from("TTL"), Box::new(TTL { milliseconds }))   
    }
}

#[async_trait]
impl super::Command for TTL {
    /// Get command body       
    async fn execute(&self, cmd: &mut Parser) -> crate::Result<Data> {
        let key = super::next_bytes!(cmd);
        super::check_end_of_param!(cmd);

        match db::DB.read().await.get(&key) {
            Some(entry) => match entry.expiration {
                Some(expiration) => match expiration.checked_duration_since(Instant::now()) {
                    Some(ttl) => {
                        match self.milliseconds {
                            super::TimeUnit::Second => {return Ok(Data::Integer(ttl.as_millis() as i64));}
                            super::TimeUnit::Millisecond => {return Ok(Data::Integer(ttl.as_secs() as i64));} 
                        }
                    },
                    None => return Ok(Data::Integer(-2)),
                },
                None => return Ok(Data::Integer(-1)),
            },
            None => return Ok(Data::Integer(-2)),
        }
    }
}
