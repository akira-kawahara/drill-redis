//! drill-redis
//! 
//! Rewrite Redis server in Rust
//! 

use drill_redis;

/// This is the entry point for the redis server.
/// 
/// arguments of run function.
/// 
/// 0.0.0.0 is bind all interfaces.
/// Port 63790 is the default for redis.
/// 
/// # Examples
/// 
/// The client can only access the server from the same machine.
/// 
/// drill_redis::server::run("localhost:6379").await
#[async_std::main]
async fn main() -> drill_redis::Result<()> {
    drill_redis::server::run("0.0.0.0:6379").await
}
