# async-locking
Async file locking using flock on unix and LockFileEx on windows.

```rust
use async_locking::AsyncLockFileExt;

let file = std::fs::File::open("target/yeet.lock")
	.expect("unable to open file");

let lock = file.lock_exclusive().await?;

// ... lock.write(...)

lock.unlock().await?;
```


## Feature flags
By default, the `tokio` feature is active.
Make sure to disable it, when using another runtime.

- `tokio`: Use the tokio runtime ([tokio::task::spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html))
- `async-std`: Use the async-std runtime ([async_std::task::spawn_blocking](https://docs.rs/async-std/latest/async_std/task/fn.spawn_blocking.html))
- `blocking`: Use the blocking thread pool ([blocking::unblock](https://docs.rs/blocking/latest/blocking/fn.unblock.html))
