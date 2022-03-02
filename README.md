# Drill-Redis
This library has been created for the purpose of evaluating Rust functionality and performance.
As such, it has not been fully tested.
The goal is to make it simple and usable as a learning tool.

# async/await

In this project, `async_std` was used for asyna/await runtime.

For more information about async/await for async_std, please refer to the following site

`Async programming in Rust with async-std`

Tutorial: Writing a chat

https://book.async.rs/tutorial/index.html

# Usage 

Build Redis server in release mode.

```
cargo build --bin dredis --release
```

Build Redis client in release mode.

```
cargo build --bin dredis-cli --release
```

Make the document with private items.
```
cargo doc --no-deps --document-private-items --open
```

Start Redis server in release mode.
```
cargo run --bin dredis --release
```

Start Redis client in release mode.
```
cargo run --bin dredis-cli --release
```

How to specify a worker thread number.

```
export ASYNC_STD_THREAD_COUNT=1
```

How to execute handlers in a single thread.
Modify listener.rs to the following.

```Rust:listener.rs line:48
task::spawn(
    to
task::spawn_local(
```

# Implemented commands

* APPEND
* DEL
* EXISTS
* EXPIRE
* GET
* GETEX - EXAT, PXAT options are not Implemented.
* PERSIST
* PEXPIER
* PING
* PTTL
* SET - EXAT, PXAT options are not Implemented.
* TTL

For more information about Redis commands, please refer to the following.

https://redis.io/commands

# Issue
Compared to the original Redis[^1], the throughput is the similar but the CPU usage is about three times higher.
(Running in a single thread by using task::spawn_local instead of task::spawn, the throughput and CPU usage were almost the same as the original Redis.)

The following commands were run as benchmarks.[^2]
```
redis-benchmark -c 50 -n 1000000 -q
```


# Contributing
Bug reports and suggestions for improvements are welcome.


# License
The source code is licensed MIT.

[^1]:Redis 2.2.0 was used for benchmarking.
[^2]:Modified to execute only PING, GET, and SET.
