use async_locking::AsyncLockFileExt;

#[cfg_attr(feature = "tokio", tokio::main)]
#[cfg_attr(feature = "async-std", async_std::main)]
#[cfg_attr(feature = "blocking", async_std::main)]
async fn main() {
	#[cfg(feature = "tokio")]
	let file = tokio::fs::File::options()
		.create(true)
		.write(true)
		.open("target/test.lock")
		.await
		.unwrap();

	#[cfg(any(feature = "async-std", feature = "blocking"))]
	let file = std::fs::File::options()
		.create(true)
		.write(true)
		.open("target/test.lock")
		.unwrap();

	println!("ready");

	let _lock = file.try_lock_exclusive()
		.unwrap()
		.unwrap();

	loop { }
}
