//! Redis commands.
//! 
//! <https://redis.io/commands>
//! 
use crate::protocol::resp::{Data, Parser};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use std::collections::HashMap;

// Refer to command modules
mod append;
mod del;
mod exists;
mod expire;
mod get;
mod getex;
mod persist;
mod ping;
mod set;
mod ttl;

/// Time unit
pub(crate) enum TimeUnit {
    Second,
    Millisecond,
}

/// Next bytes macro.
/// If command stream is EOF, return OK(error response).
macro_rules! next_bytes {
    ($cmd:expr) => {
        match $cmd.next_bytes()? {
            Some(key) => key,
            None => return Ok(Data::error("wrong number of arguments for command")),
        }
    };
}
/// Next u64 macro.
/// If command stream is EOF, return OK(error response).
macro_rules! next_u64 {
    ($cmd:expr) => {
        match $cmd.next_u64()? {
            Some(key) => key,
            None => return Ok(Data::error("wrong number of arguments for command")),
        }
    };
}
/// Check end of param macro.
/// If command stream isn't EOF, return OK(error response).
macro_rules! check_end_of_param {
    ($cmd:expr) => {
        match $cmd.next_bytes()? {
            None => {}
            _ => return Ok(Data::error("wrong number of arguments for command")),
        }
    };
}

// Allow macros to be used outside of this module.
pub(crate) use check_end_of_param;
pub(crate) use next_bytes;
pub(crate) use next_u64;

/// Command type definition
pub(crate) type Cmd = Box<dyn Command + Send + Sync>;
/// Commnad manager singleton
static COMMANDS: Lazy<CommandManager> = Lazy::new(|| CommandManager::new());
/// Command manager
struct CommandManager {
    commands: HashMap<String, Cmd>,
}

impl CommandManager {
    fn new() -> Self {
        CommandManager {
            //Register command here.
            commands: HashMap::from([
                get::command(),
                set::command(),
                ttl::command(TimeUnit::Second),
                ttl::command(TimeUnit::Millisecond),
                del::command(),
                exists::command(),
                ping::command(),
                persist::command(),
                append::command(),
                getex::command(),
                expire::command(TimeUnit::Second),
                expire::command(TimeUnit::Millisecond),
            ]),
        }
    }
    /// Execute command.
    async fn execute(&self, cmd: &mut Parser) -> Data {
        match cmd.next_string() {
            Ok(Some(cmd_name)) => {
                if let Some(cmd_func) = self.commands.get(&cmd_name) {
                    match cmd_func.execute(cmd).await {
                        Ok(response) => response,
                        Err(e) => Data::error(&format!("{}", e)),
                    }
                } else {
                    Data::error(&format!("Unknown or disabled command '{}'", cmd_name))
                }
            }
            Err(e) => Data::error(&format!("{}", e)),
            _ => Data::error("protcol error"),
        }
    }
}

/// Handle each command with the same interface.
#[async_trait]
pub(crate) trait Command {
    async fn execute(&self, cmd: &mut Parser) -> crate::Result<Data>;
}

/// Execute command.
pub(crate) async fn execute(cmd: Data) -> crate::Result<Data> {
    if let Some(mut parser) = Parser::new(cmd) {
        let response = COMMANDS.execute(&mut parser).await;
        Ok(response)
    } else {
        Ok(Data::error("protocol error"))
    }
}
