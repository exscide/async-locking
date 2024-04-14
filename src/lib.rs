//! Async implementation of file locking using flock on unix and LockFileEx on windows.
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
use windows::*;

#[cfg(not(windows))]
mod unix;
#[cfg(not(windows))]
use unix::*;


/// An extension trait for any [std::fs::File] like type that provides async file locking methods.
pub trait AsyncLockFileExt: AsDescriptor {
	/// Asynchronously wait to obtain a shared lock
	fn lock_shared(self) -> impl Future<Output = std::io::Result<Lock<Self>>> + Send where Self: Sized;
	/// Asynchronously wait to obtain an exclusive lock
	fn lock_exclusive(self) -> impl Future<Output = std::io::Result<Lock<Self>>> + Send where Self: Sized;
	/// Try to obtain a shared lock
	fn try_lock_shared(&self) -> std::io::Result<()>;
	/// Try to obtain an exclusive lock
	fn try_lock_exclusive(&self) -> std::io::Result<()>;
}



impl<T: AsDescriptor + Send + 'static> AsyncLockFileExt for T {
	fn lock_shared(self) -> impl Future<Output = std::io::Result<Lock<Self>>> + Send {
		async move {
			#[cfg(feature = "tokio")]
			let spawn = tokio::task::spawn_blocking;
			#[cfg(feature = "async-std")]
			let spawn = async_std::task::spawn_blocking;
			#[cfg(feature = "blocking")]
			let spawn = blocking::unblock;

			let res = spawn(move || lock_shared(self))
				.await;

			#[cfg(feature = "tokio")]
			let res = res.unwrap();

			res.map(|file| Lock::new(file))
		}
	}

	fn lock_exclusive(self) -> impl Future<Output = std::io::Result<Lock<Self>>> + Send {
		async move {
			#[cfg(feature = "tokio")]
			let spawn = tokio::task::spawn_blocking;
			#[cfg(feature = "async-std")]
			let spawn = async_std::task::spawn_blocking;
			#[cfg(feature = "blocking")]
			let spawn = blocking::unblock;

			let res = spawn(move || lock_exclusive(self))
				.await;

			#[cfg(feature = "tokio")]
			let res = res.unwrap();

			res.map(|file| Lock::new(file))
		}
	}

	fn try_lock_shared(&self) -> std::io::Result<()> {
		try_lock_shared(self)
	}

	fn try_lock_exclusive(&self) -> std::io::Result<()> {
		try_lock_exclusive(self)
	}
}


pub struct Lock<T: AsDescriptor> {
	file: T,
}

impl<T: AsDescriptor> Lock<T> {
	pub(crate) fn new(file: T) -> Self {
		Self { file }
	}

	pub fn unlock(self) -> T {
		let _ = self.unlock_ref();
		let file = unsafe { std::mem::transmute_copy(&self.file) };
		std::mem::forget(self);
		file
	}

	pub(crate) fn unlock_ref(&self) -> std::io::Result<()> {
		unlock(&self.file)
	}
}

impl<T: AsDescriptor> Drop for Lock<T> {
	fn drop(&mut self) {
		let _ = self.unlock_ref();
	}
}

impl<T: AsDescriptor> std::ops::Deref for Lock<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.file
	}
}

impl<T: AsDescriptor> std::ops::DerefMut for Lock<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.file
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[cfg(feature = "tokio")]
	#[tokio::test]
	async fn test() {
		let file = tokio::fs::File::options()
			.create(true)
			.read(true)
			.write(true)
			.open("target/ok")
			.await
			.unwrap();

		let fut = file.lock_shared();

		fut.await.unwrap();
	}
}
