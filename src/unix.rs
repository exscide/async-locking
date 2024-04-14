use rustix::fs::{ flock, FlockOperation };
use std::{io::ErrorKind, os::unix::io::AsFd};


/// Catchall trait for files on unix
pub trait AsDescriptor: AsFd + Send + 'static {}
impl<T: AsFd + Send + 'static> AsDescriptor for T {}


pub(crate) fn lock_shared<F: AsFd>(file: F) -> std::io::Result<F> {
	lock_file(&file, FlockOperation::LockShared)?;
	Ok(file)
}

pub(crate) fn lock_exclusive<F: AsFd>(file: F) -> std::io::Result<F> {
	lock_file(&file, FlockOperation::LockExclusive)?;
	Ok(file)
}

pub(crate) fn try_lock_shared<F: AsFd>(file: &F) -> std::io::Result<Option<()>> {
	let res = lock_file(file, FlockOperation::NonBlockingLockShared);

	if let Err(e) = &res {
		if let ErrorKind::WouldBlock = e.kind() {
			return Ok(None);
		}
	}

	res.map(|_| Some(()))
}

pub(crate) fn try_lock_exclusive<F: AsFd>(file: &F) -> std::io::Result<Option<()>> {
	let res = lock_file(file, FlockOperation::NonBlockingLockExclusive);

	if let Err(e) = &res {
		if let ErrorKind::WouldBlock = e.kind() {
			return Ok(None);
		}
	}

	res.map(|_| Some(()))
}

pub(crate) fn unlock<F: AsFd>(file: F) -> std::io::Result<F> {
	unlock_ref(&file).map(|_| file)
}

pub(crate) fn unlock_ref<F: AsFd>(file: &F) -> std::io::Result<()> {
	lock_file(file, FlockOperation::Unlock)
}

fn lock_file<F: AsFd>(file: F, op: FlockOperation) -> std::io::Result<()> {
	flock(file, op).map_err(|e| e.into())
}
