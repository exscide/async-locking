use std::{ io::{ Error, ErrorKind }, os::fd::AsRawFd };

use libc::{c_int, LOCK_EX, LOCK_NB, LOCK_SH, LOCK_UN};


/// Catchall trait for [File](std::fs::File) like types
pub trait AsDescriptor: AsRawFd + Send + 'static {}
impl<T: AsRawFd + Send + 'static> AsDescriptor for T {}


pub(crate) fn lock_shared<F: AsRawFd>(file: F) -> std::io::Result<F> {
	lock_file(&file, LOCK_SH)?;
	Ok(file)
}

pub(crate) fn lock_exclusive<F: AsRawFd>(file: F) -> std::io::Result<F> {
	lock_file(&file, LOCK_EX)?;
	Ok(file)
}

pub(crate) fn try_lock_shared<F: AsRawFd>(file: &F) -> std::io::Result<Option<()>> {
	let res = lock_file(file, LOCK_SH | LOCK_NB);

	if let Err(e) = &res {
		if let ErrorKind::WouldBlock = e.kind() {
			return Ok(None);
		}
	}

	res.map(|_| Some(()))
}

pub(crate) fn try_lock_exclusive<F: AsRawFd>(file: &F) -> std::io::Result<Option<()>> {
	let res = lock_file(file, LOCK_EX | LOCK_NB);

	if let Err(e) = &res {
		if let ErrorKind::WouldBlock = e.kind() {
			return Ok(None);
		}
	}

	res.map(|_| Some(()))
}

pub(crate) fn unlock<F: AsRawFd>(file: F) -> std::io::Result<F> {
	unlock_ref(&file).map(|_| file)
}

pub(crate) fn unlock_ref<F: AsRawFd>(file: &F) -> std::io::Result<()> {
	lock_file(file, LOCK_UN)
}

fn lock_file<F: AsRawFd>(file: &F, op: c_int) -> std::io::Result<()> {
	let res = unsafe {
		libc::flock(file.as_raw_fd(), op)
	};

	match res {
		0 => Ok(()),
		_ => Err(Error::last_os_error())
	}
}
