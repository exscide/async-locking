use super::*;
use std::{ io::{ BufRead, BufReader }, process::{ Child, ChildStdout, ExitStatus, Stdio }, sync::mpsc::channel, time::Duration };

struct Process {
	inner: Child,
	stdout: BufReader<ChildStdout>,
}

impl Process {
	pub fn new(command: &str, args: &[&str]) -> std::io::Result<Self> {
		let mut child = std::process::Command::new(command)
			.args(args)
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()?;

		Ok(Self {
			stdout: BufReader::new(child.stdout.take().unwrap()),
			inner: child,
		})
	}

	pub fn wait_for(&mut self, cmd: &str) -> std::io::Result<()> {
		loop {
			let mut buf = String::new();
			self.stdout.read_line(&mut buf)?;
			if buf.contains(cmd) {
				break;
			}
		}

		Ok(())
	}

	pub fn kill(&mut self) -> std::io::Result<()> {
		self.inner.kill()
	}

	pub fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
		self.inner.try_wait()
	}
}

impl Drop for Process {
	fn drop(&mut self) {
		let _ = self.inner.kill();
	}
}

fn blocker() -> Process {
	Process::new("cargo", &[
			"run",
			"--example",
			"block"
		])
		.unwrap()
}

async fn open_file(path: &str) -> std::fs::File {
	std::fs::File::options()
		.create(true)
		.write(true)
		.open(path)
		.unwrap()
}

#[cfg_attr(feature = "tokio", tokio::test(flavor = "multi_thread", worker_threads = 4))]
#[cfg_attr(feature = "async-std", async_std::test)]
#[cfg_attr(feature = "blocking", async_std::test)]
async fn test_lock_interprocess() {
	use std::time::Duration;


	// -- other thread is blocking --

	let mut blck = blocker();
	blck.wait_for("ready").unwrap();

	let mut file = open_file("target/test.lock").await;

	file.try_lock_exclusive_ref().unwrap().ok_or(()).expect_err("File should be exclusively locked");
	file.try_lock_shared_ref().unwrap().ok_or(()).expect_err("File should be exclusively locked");

	blck.kill().unwrap();

	// -- other thread stopped blocking --

	std::thread::sleep(Duration::from_millis(200));

	let l = file.try_lock_exclusive_ref().unwrap().unwrap();
	drop(l);

	#[cfg(feature = "tokio")]
	let timeout = tokio::time::timeout;
	#[cfg(any(feature = "async-std", feature = "blocking"))]
	let timeout = async_std::future::timeout;

	let lock = timeout(Duration::from_secs(2), file.lock_exclusive_ref())
		.await
		.unwrap()
		.unwrap();


	// -- this thread is blocking --

	let mut blck = blocker();

	let code = blck.try_wait().unwrap();

	if code.is_some() {
		panic!("expected panic");
	}

	lock.unlock().unwrap();

	#[cfg(feature = "tokio")]
	os_test().await;
}


#[cfg_attr(feature = "tokio", tokio::test)]
#[cfg_attr(feature = "async-std", async_std::test)]
#[cfg_attr(feature = "blocking", async_std::test)]
async fn test_lock_current_process() {
	let mut file = open_file("target/test2.lock").await;
	let mut file2 = open_file("target/test2.lock").await;

	let lock = file.try_lock_exclusive_ref()
		.unwrap()
		.unwrap();
	assert!(file2.try_lock_exclusive_ref().unwrap().is_none());

	lock.unlock().unwrap();

	let lock = file2.try_lock_exclusive_ref()
		.unwrap()
		.unwrap();

	assert!(file.try_lock_exclusive_ref().unwrap().is_none());

	lock.unlock().unwrap();

	let lock = file.try_lock_shared_ref()
		.unwrap()
		.unwrap();
	assert!(file2.try_lock_shared_ref().unwrap().is_some());
	assert!(file2.try_lock_exclusive_ref().unwrap().is_none());

	lock.unlock().unwrap();
}


#[cfg(feature = "tokio")]
async fn os_test() {
	// lock the file
	let mut blck = blocker();
	blck.wait_for("ready").unwrap();

	let mut f = std::fs::File::options()
		.create(true)
		.write(true)
		.open("target/test.lock")
		.unwrap();

	assert!(f.try_lock_exclusive_ref().unwrap().is_none());

	let fd = f.as_descriptor();

	// it seems like the drop call blocks until the lock call is done,
	// ensure this behavior across platforms

	let (s, r) = channel();

	let _waiter = tokio::spawn(async move {
		s.send(()).unwrap();
		println!("waiting");
		tokio::task::spawn_blocking(move || lock_exclusive(fd)).await.unwrap()
	});

	let dropper = tokio::spawn(async move {
		r.recv().unwrap();
		std::thread::sleep(Duration::from_secs(1));
		println!("dropping file");
		drop(f);
		println!("dropped file");
	});

	tokio::time::timeout(Duration::from_secs(2), async { println!("waiting for dropper"); dropper.await })
		.await
		.expect_err("drop should wait for lock");

	println!("killing blocker");
	blck.kill().unwrap();

	// it's probably undefined behavior what happens to the waiter,
	// the call seems to stall on windows and sometimes stall on macos

	// trying to lock a dropped file should error
	#[cfg(not(windows))]
	let err = libc::EBADF;
	#[cfg(windows)]
	let err = windows_sys::Win32::Foundation::ERROR_INVALID_HANDLE as i32;
	assert_eq!(lock_exclusive(fd).map_err(|e| e.raw_os_error()), Err(Some(err)));
}
