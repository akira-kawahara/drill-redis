//! drill-redis-cli
//! 
//! Rewrite Redis client in Rust
//! 
use drill_redis;

/// This is the entry point for the redis client.
/// 
/// arguments of run function.
/// 
/// 127.0.0.1 is server address.
/// Port 63790 is the default for redis.
/// 
/// # Examples
/// 
/// drill_redis::client::run("127.0.0.1:6379").await
#[async_std::main]
async fn main() -> drill_redis::Result<()> {
    drill_redis::client::run("127.0.0.1:6379").await
}