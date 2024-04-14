use std::time::Duration;

use async_locking::AsyncLockFileExt;

#[tokio::main]
async fn main() {
	let mut file = tokio::fs::File::options()
		.create(true)
		.read(true)
		.write(true)
		.open("target/test.lock")
		.await
		.unwrap();

	println!("trying to block");

	let _lock = file.try_lock_exclusive()
		.unwrap()
		.unwrap();

	println!("blocking");

	std::thread::sleep(Duration::from_secs(10));
}
