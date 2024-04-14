use rustix::fs::{ flock, FlockOperation };
use std::os::unix::io::AsFd;


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

pub(crate) fn try_lock_shared<F: AsFd>(file: F) -> std::io::Result<()> {
	lock_file(file, FlockOperation::NonBlockingLockShared)
}

pub(crate) fn try_lock_exclusive<F: AsFd>(file: F) -> std::io::Result<()> {
	lock_file(file, FlockOperation::NonBlockingLockExclusive)
}

pub(crate) fn unlock<F: AsFd>(file: F) -> std::io::Result<()> {
	lock_file(file, FlockOperation::Unlock)
}

fn lock_file<F: AsFd>(file: F, op: FlockOperation) -> std::io::Result<()> {
	flock(file, op).map_err(|e| e.into())
}
