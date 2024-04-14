use async_locking::AsyncLockFileExt;

#[tokio::main]
async fn main() {
	let file = tokio::fs::File::options()
		.create(true)
		.read(true)
		.write(true)
		.open("target/test.lock")
		.await
		.unwrap();

	let _lock = file.try_lock_exclusive().unwrap();

	loop { }
}
