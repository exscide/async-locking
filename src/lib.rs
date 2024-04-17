//! Async file locking using [flock](https://man7.org/linux/man-pages/man2/flock.2.html) on unix and [LockFileEx](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-lockfileex) on windows.
//! 
//! ```
//! use async_locking::AsyncLockFileExt;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! 	let file = std::fs::File::options()
//! 		.create(true)
//! 		.write(true)
//! 		.open("target/yeet.lock")
//! 		.expect("unable to open file");
//! 
//! 	let lock = file.lock_exclusive().await?;
//! 
//! 	// ... lock.write(...)
//! 
//! 	lock.unlock().await?;
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
	fn lock_shared(self) -> impl Future<Output = std::io::Result<Lock<Self>>> + Send where Self: Sized + 'static;

	/// Asynchronously wait to obtain an exclusive lock
	fn lock_exclusive(self) -> impl Future<Output = std::io::Result<Lock<Self>>> + Send where Self: Sized + 'static;

	/// Try to obtain a shared lock
	fn try_lock_shared<'a>(&'a mut self) -> std::io::Result<Option<LockRef<'a, Self>>> where Self: Sized + 'static {
		try_lock_shared(self).map(|f| f.map(|_| LockRef::new(self)))
	}

	/// Try to obtain an exclusive lock
	fn try_lock_exclusive<'a>(&'a mut self) -> std::io::Result<Option<LockRef<'a, Self>>> where Self: Sized + 'static {
		try_lock_exclusive(self).map(|f| f.map(|_| LockRef::new(self)))
	}
}



impl<T: AsDescriptor> AsyncLockFileExt for T {
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
}


/// Guard that holds a locked file.
/// 
/// It automatically unlocks the file on drop, but it can be manually unlocked using [Lock::unlock].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lock<T: AsDescriptor> {
	file: T,
}

impl<T: AsDescriptor> Lock<T> {
	pub(crate) fn new(file: T) -> Self {
		Self { file }
	}

	// TODO: async unlock useful?
	/// Asynchronously unlock the file
	pub async fn unlock(self) -> std::io::Result<T> {
		#[cfg(feature = "tokio")]
		let spawn = tokio::task::spawn_blocking;
		#[cfg(feature = "async-std")]
		let spawn = async_std::task::spawn_blocking;
		#[cfg(feature = "blocking")]
		let spawn = blocking::unblock;

		// SAFETY: we're manually dropping the lock
		let file = unsafe { std::ptr::read(&self.file) };

		let res = spawn(move || unlock(file))
			.await;

		#[cfg(feature = "tokio")]
		let res = res.unwrap();

		std::mem::forget(self);

		res
	}

	pub unsafe fn unlock_ref(&self) -> std::io::Result<()> {
		unlock_ref(&self.file).map(|_| ())
	}
}

impl<T: AsDescriptor> Drop for Lock<T> {
	fn drop(&mut self) {
		// SAFETY: can only be called once, since the safe unlock method takes ownership
		let _ = unsafe { self.unlock_ref() };
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
		unlock_ref(self.file)?;
		std::mem::forget(self);
		Ok(())
	}

	pub unsafe fn unlock_ref(&self) -> std::io::Result<()> {
		unlock_ref(self.file)
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
