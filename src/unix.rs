use std::{ io::{ Error, ErrorKind }, os::fd::{ AsRawFd, RawFd } };

use libc::{c_int, LOCK_EX, LOCK_NB, LOCK_SH, LOCK_UN};


pub type Descriptor = RawFd;

/// Catchall trait for [File](std::fs::File) like types
pub trait AsDescriptor: Send + 'static {
	fn as_descriptor(&self) -> Descriptor;
}

impl<T: AsRawFd + Send + 'static> AsDescriptor for T {
	fn as_descriptor(&self) -> Descriptor {
		self.as_raw_fd()
	}
}


pub(crate) fn lock_shared(file: Descriptor) -> std::io::Result<()> {
	lock_file(file, LOCK_SH)
}

pub(crate) fn lock_exclusive(file: Descriptor) -> std::io::Result<()> {
	lock_file(file, LOCK_EX)
}

pub(crate) fn try_lock_shared(file: Descriptor) -> std::io::Result<bool> {
	let res = lock_file(file, LOCK_SH | LOCK_NB);

	if let Err(e) = &res {
		if let ErrorKind::WouldBlock = e.kind() {
			return Ok(false);
		}
	}

	res.map(|_| true)
}

pub(crate) fn try_lock_exclusive(file: Descriptor) -> std::io::Result<bool> {
	let res = lock_file(file, LOCK_EX | LOCK_NB);

	if let Err(e) = &res {
		if let ErrorKind::WouldBlock = e.kind() {
			return Ok(false);
		}
	}

	res.map(|_| true)
}

pub(crate) fn unlock(file: Descriptor) -> std::io::Result<()> {
	lock_file(file, LOCK_UN)
}

fn lock_file(file: Descriptor, op: c_int) -> std::io::Result<()> {
	let res = unsafe {
		libc::flock(file, op)
	};

	match res {
		0 => Ok(()),
		_ => Err(Error::last_os_error())
	}
}
