use async_locking::AsyncLockFileExt;

#[cfg_attr(feature = "tokio", tokio::main)]
#[cfg_attr(feature = "async-std", async_std::main)]
async fn main() {
	#[cfg(feature = "tokio")]
	let mut file = tokio::fs::File::options()
		.create(true)
		.write(true)
		.open("target/test.lock")
		.await
		.unwrap();

	#[cfg(feature = "async-std")]
	let mut file = async_std::fs::OpenOptions::new()
		.create(true)
		.write(true)
		.open("target/test.lock")
		.await
		.unwrap();

	println!("ready");

	let _lock = file.try_lock_exclusive()
		.unwrap()
		.unwrap();

	loop { }
}
