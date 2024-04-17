use windows_sys::Win32::{ Foundation::ERROR_LOCK_VIOLATION, Storage::FileSystem::{ LockFileEx, UnlockFile, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY } };
use std::os::windows::io::AsRawHandle;

pub type Descriptor = isize;

/// Catchall trait for [File](std::fs::File) like types
pub trait AsDescriptor: AsRawHandle + Send + 'static {
	fn as_descriptor(&self) -> Descriptor;
}

impl<T: AsRawHandle + Send + 'static> AsDescriptor for T {
	fn as_descriptor(&self) -> Descriptor {
		self.as_raw_handle() as Descriptor
	}
}


pub(crate) fn lock_shared(file: Descriptor) -> std::io::Result<()> {
	lock_file(file, 0)
}

pub(crate) fn lock_exclusive(file: Descriptor) -> std::io::Result<()> {
	lock_file(file, LOCKFILE_EXCLUSIVE_LOCK)
}

pub(crate) fn try_lock_shared(file: Descriptor) -> std::io::Result<Option<()>> {
	let res = lock_file(file, LOCKFILE_FAIL_IMMEDIATELY);

	if let Err(Some(code)) = res.as_ref().map_err(|e| e.raw_os_error()) {
		if code == ERROR_LOCK_VIOLATION as i32 {
			return Ok(None);
		}
	}

	res.map(|_| Some(()))
}

pub(crate) fn try_lock_exclusive(file: Descriptor) -> std::io::Result<Option<()>> {
	let res = lock_file(file, LOCKFILE_FAIL_IMMEDIATELY | LOCKFILE_EXCLUSIVE_LOCK);

	if let Err(Some(code)) = res.as_ref().map_err(|e| e.raw_os_error()) {
		if code == ERROR_LOCK_VIOLATION as i32 {
			return Ok(None);
		}
	}

	res.map(|_| Some(()))
}

fn lock_file(file: Descriptor, flags: u32) -> Result<(), std::io::Error> {
	let ret = unsafe {
		let mut overlapped = std::mem::zeroed();
		LockFileEx(
			file,
			flags,
			0,
			!0,
			!0,
			&mut overlapped,
		)
	};

	match ret {
		0 => Err(std::io::Error::last_os_error()),
		_ => Ok(())
	}
}

pub(crate) fn unlock(file: Descriptor) -> std::io::Result<()> {
	let ret = unsafe {
		UnlockFile(
			file,
			0,
			0,
			!0,
			!0
		)
	};

	match ret {
		0 => Err(std::io::Error::last_os_error()),
		_ => Ok(())
	}
}
