//! Async file locking using [flock](https://man7.org/linux/man-pages/man2/flock.2.html) on unix and [LockFileEx](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-lockfileex) on windows.
//! 
//! ```
//! use async_locking::AsyncLockFileExt;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! 	let mut file = std::fs::File::options()
//! 		.create(true)
//! 		.write(true)
//! 		.open("target/yeet.lock")
//! 		.expect("unable to open file");
//! 
//! 	let lock = file.lock_exclusive().await?;
//! 
//! 	// ... lock.write(...)
//! 
//! 	lock.unlock()?;
//! 
//! 	Ok(())
//! }
//! ```
//! 
//! ## Feature flags
//! By default, the `tokio` feature is active.
//! Make sure to disable it, when using another runtime.
//! 
//! 
//! - `tokio`: Use the tokio runtime ([tokio::task::spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html))
//! - `async-std`: Use the async-std runtime ([async_std::task::spawn_blocking](https://docs.rs/async-std/latest/async_std/task/fn.spawn_blocking.html))
//! - `blocking`: Use the blocking thread pool ([blocking::unblock](https://docs.rs/blocking/latest/blocking/fn.unblock.html))

#[cfg(any(
	all(feature = "tokio", feature = "async-std"),
	all(feature = "tokio", feature = "blocking"),
	all(feature = "async-std", feature = "blocking"),
))]
compile_error!("feature \"tokio\", \"async-std\" and \"blocking\" are mutually exclusive");

use std::future::Future;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(not(windows))]
mod unix;
#[cfg(not(windows))]
pub use unix::*;


/// Extension trait for [File](std::fs::File) like types that provides async file locking methods.
pub trait AsyncLockFileExt: AsDescriptor {
	/// Asynchronously wait to obtain a shared lock
	fn lock_shared(&mut self) -> impl Future<Output = std::io::Result<LockRef<Self>>> + Send where Self: Sized + 'static;

	/// Asynchronously wait to obtain an exclusive lock
	fn lock_exclusive(&mut self) -> impl Future<Output = std::io::Result<LockRef<Self>>> + Send where Self: Sized + 'static;

	/// Try to obtain a shared lock
	fn try_lock_shared<'a>(&'a mut self) -> std::io::Result<Option<LockRef<'a, Self>>> where Self: Sized + 'static {
		try_lock_shared(self.as_descriptor()).map(|f| f.map(|_| LockRef::new(self)))
	}

	/// Try to obtain an exclusive lock
	fn try_lock_exclusive<'a>(&'a mut self) -> std::io::Result<Option<LockRef<'a, Self>>> where Self: Sized + 'static {
		try_lock_exclusive(self.as_descriptor()).map(|f| f.map(|_| LockRef::new(self)))
	}
}



impl<T: AsDescriptor> AsyncLockFileExt for T {
	fn lock_shared(&mut self) -> impl Future<Output = std::io::Result<LockRef<Self>>> + Send {
		async move {
			#[cfg(feature = "tokio")]
			let spawn = tokio::task::spawn_blocking;
			#[cfg(feature = "async-std")]
			let spawn = async_std::task::spawn_blocking;
			#[cfg(feature = "blocking")]
			let spawn = blocking::unblock;

			let desc = self.as_descriptor();

			let res = spawn(move || lock_shared(desc))
				.await;

			#[cfg(feature = "tokio")]
			let res = res.unwrap();

			res.map(|_| LockRef::new(self))
		}
	}

	fn lock_exclusive(&mut self) -> impl Future<Output = std::io::Result<LockRef<Self>>> + Send {
		async move {
			#[cfg(feature = "tokio")]
			let spawn = tokio::task::spawn_blocking;
			#[cfg(feature = "async-std")]
			let spawn = async_std::task::spawn_blocking;
			#[cfg(feature = "blocking")]
			let spawn = blocking::unblock;

			let desc = self.as_descriptor();

			let res = spawn(move || lock_exclusive(desc))
				.await;

			#[cfg(feature = "tokio")]
			let res = res.unwrap();

			res.map(|_| LockRef::new(self))
		}
	}
}


/// Guard that holds a reference to a locked file.
/// 
/// It automatically unlocks the file on drop, but it can be manually unlocked using [LockRef::unlock].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LockRef<'a, T: AsDescriptor> {
	file: &'a mut T,
}

impl<'a, T: AsDescriptor> LockRef<'a, T> {
	pub(crate) fn new(file: &'a mut T) -> Self {
		Self { file }
	}

	/// Unlock the file
	pub fn unlock(self) -> std::io::Result<()> {
		unlock(self.file.as_descriptor())?;
		std::mem::forget(self);
		Ok(())
	}

	pub unsafe fn unlock_ref(&self) -> std::io::Result<()> {
		unlock(self.file.as_descriptor())
	}
}

impl<'a, T: AsDescriptor> Drop for LockRef<'a, T> {
	fn drop(&mut self) {
		// SAFETY: can only be called once, since the safe unlock method takes ownership
		let _ = unsafe { self.unlock_ref() };
	}
}

impl<'a, T: AsDescriptor> std::ops::Deref for LockRef<'a, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.file
	}
}

impl<'a, T: AsDescriptor> std::ops::DerefMut for LockRef<'a, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.file
	}
}


#[cfg(test)]
mod tests;
