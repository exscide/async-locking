use windows_sys::Win32::{ Foundation::ERROR_LOCK_VIOLATION, Storage::FileSystem::{ LockFileEx, UnlockFile, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY } };
use std::os::windows::io::AsRawHandle;


/// Catchall trait for files on windows
pub trait AsDescriptor: AsRawHandle + Send + 'static {}
impl<T: AsRawHandle + Send + 'static> AsDescriptor for T {}


pub(crate) fn lock_shared<F: AsRawHandle>(file: F) -> std::io::Result<F> {
	lock_file(file.as_raw_handle() as isize, 0)?;
	Ok(file)
}

pub(crate) fn lock_exclusive<F: AsRawHandle>(file: F) -> std::io::Result<F> {
	lock_file(file.as_raw_handle() as isize, LOCKFILE_EXCLUSIVE_LOCK)?;
	Ok(file)
}

pub(crate) fn try_lock_shared<F: AsRawHandle>(file: &F) -> std::io::Result<Option<()>> {
	let res = lock_file(file.as_raw_handle() as isize, LOCKFILE_FAIL_IMMEDIATELY);

	if let Err(Some(code)) = res.as_ref().map_err(|e| e.raw_os_error()) {
		if code == ERROR_LOCK_VIOLATION as i32 {
			return Ok(None);
		}
	}

	res.map(|_| Some(()))
}

pub(crate) fn try_lock_exclusive<F: AsRawHandle>(file: &F) -> std::io::Result<Option<()>> {
	let res = lock_file(file.as_raw_handle() as isize, LOCKFILE_FAIL_IMMEDIATELY | LOCKFILE_EXCLUSIVE_LOCK);

	if let Err(Some(code)) = res.as_ref().map_err(|e| e.raw_os_error()) {
		if code == ERROR_LOCK_VIOLATION as i32 {
			return Ok(None);
		}
	}

	res.map(|_| Some(()))
}

fn lock_file(file: isize, flags: u32) -> Result<(), std::io::Error> {
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

	if ret == 0 {
		Err(std::io::Error::last_os_error())
	} else {
		Ok(())
	}
}

pub(crate) fn unlock<F: AsRawHandle>(file: F) -> Result<F, std::io::Error> {
	unlock_ref(&file).map(|_| file)
}

pub(crate) fn unlock_ref<F: AsRawHandle>(file: &F) -> Result<(), std::io::Error> {
	let ret = unsafe {
		UnlockFile(file.as_raw_handle() as isize, 0, 0, !0, !0)
	};
	if ret == 0 {
		Err(std::io::Error::last_os_error())
	} else {
		Ok(())
	}
}
