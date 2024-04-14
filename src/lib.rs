use std::future::Future;


#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows::*;

pub trait AsyncFileExt {
	fn lock_shared(&self) -> impl Future<Output = std::io::Result<()>> + Send;
	fn lock_exclusive(&self) -> impl Future<Output = std::io::Result<()>> + Send;
	fn try_lock_shared(&self) -> std::io::Result<()>;
	fn try_lock_exclusive(&self) -> std::io::Result<()>;
}


#[cfg(feature = "tokio")]
mod impl_tokio {
	use super::*;
	use std::os::windows::io::{ AsHandle, AsRawHandle };

	impl AsyncFileExt for tokio::fs::File {
		fn lock_shared(&self) -> impl Future<Output = Result<(), std::io::Error>> + Send {
			let handle = self.as_handle().as_raw_handle() as isize;
			async move {
				tokio::task::spawn_blocking(move || lock_shared(handle)).await.unwrap()
			}
		}

		fn lock_exclusive(&self) -> impl Future<Output = Result<(), std::io::Error>> + Send {
			let handle = self.as_raw_handle() as isize;
			async move {
				tokio::task::spawn_blocking(move || lock_exclusive(handle)).await.unwrap()
			}
		}

		fn try_lock_shared(&self) -> std::io::Result<()> {
			try_lock_shared(self.as_raw_handle() as isize)
		}

		fn try_lock_exclusive(&self) -> std::io::Result<()> {
			try_lock_exclusive(self.as_raw_handle() as isize)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test() {
		let file = tokio::fs::File::options()
			.create(true)
			.read(true)
			.write(true)
			.open("ok")
			.await
			.unwrap();

		let fut = file.lock_shared();

		fut.await.unwrap();
	}
}
