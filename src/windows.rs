use windows_sys::Win32::Storage::FileSystem::{ LockFileEx, UnlockFile, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY };


pub(crate) fn lock_shared(file: isize) -> std::io::Result<()> {
	lock_file(file, 0)
}

pub(crate) fn lock_exclusive(file: isize) -> std::io::Result<()> {
	lock_file(file, LOCKFILE_EXCLUSIVE_LOCK)
}

pub(crate) fn try_lock_shared(file: isize) -> std::io::Result<()> {
	lock_file(file, LOCKFILE_FAIL_IMMEDIATELY)
}

pub(crate) fn try_lock_exclusive(file: isize) -> std::io::Result<()> {
	lock_file(file, LOCKFILE_FAIL_IMMEDIATELY | LOCKFILE_EXCLUSIVE_LOCK)
}

fn lock_file(file: isize, flags: u32) -> Result<(), std::io::Error> {
	unsafe {
		let mut overlapped = std::mem::zeroed();
		let ret = LockFileEx(
			file,
			flags,
			0,
			!0,
			!0,
			&mut overlapped,
		);
		if ret == 0 {
			Err(std::io::Error::last_os_error())
		} else {
			Ok(())
		}
	}
}

pub(crate) fn unlock(file: isize) -> Result<(), std::io::Error> {
	unsafe {
		let ret = UnlockFile(file as isize, 0, 0, !0, !0);
		if ret == 0 {
			Err(std::io::Error::last_os_error())
		} else {
			Ok(())
		}
	}
}
