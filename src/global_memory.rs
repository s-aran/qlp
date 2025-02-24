use std::fmt::Display;

use windows::Win32::{
    Foundation::{GetLastError, GlobalFree, HANDLE, HGLOBAL},
    System::Memory::{
        GMEM_MOVEABLE, GMEM_ZEROINIT, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
    },
};

pub struct GlobalMemory {
    handle: HANDLE,
    allocated: bool,
    locked: bool,
    h_global: HGLOBAL,
    ptr: *mut std::ffi::c_void,
}

impl GlobalMemory {
    pub fn new() -> Self {
        Self {
            handle: HANDLE::default(),
            h_global: HGLOBAL::default(),
            ptr: std::ptr::null_mut(),
            allocated: false,
            locked: false,
        }
    }

    pub fn get_handle(&self) -> HANDLE {
        self.handle
    }

    pub fn get_global(&self) -> HGLOBAL {
        self.h_global
    }

    pub fn is_allocated(&self) -> bool {
        self.allocated
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn size(&self) -> usize {
        unsafe { GlobalSize(self.h_global) }
    }

    pub fn alloc(&mut self, size: usize) -> Result<*mut std::ffi::c_void, String> {
        if self.is_locked() {
            return Err("already locked.".to_owned());
        }

        if self.is_allocated() {
            return Err("already allocated.".to_owned());
        }

        let h_global = match unsafe { GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, size) } {
            Ok(h) => h,
            Err(e) => return Err(format!("GlobalAlloc() failed: {}", e.to_string())),
        };

        self.allocated = true;

        self.lock(h_global)
    }

    pub fn alloc_without_free(&mut self, size: usize) -> Result<*mut std::ffi::c_void, String> {
        let result = self.alloc(size);

        // avoid call free() pm drop
        self.allocated = false;

        result
    }

    pub fn lock(&mut self, h_global: HGLOBAL) -> Result<*mut std::ffi::c_void, String> {
        if self.is_locked() {
            return Err("already locked.".to_owned());
        }

        self.ptr = unsafe { GlobalLock(h_global) };
        if self.ptr.is_null() {
            return Err("GlobalLock() failed.".to_owned());
        }

        self.locked = true;
        self.h_global = h_global;

        Ok(self.ptr)
    }

    pub fn lock_by_handle(&mut self, handle: HANDLE) -> Result<*mut std::ffi::c_void, String> {
        self.lock(HGLOBAL(handle.0))
    }

    pub fn unlock(&mut self) -> Result<(), String> {
        if !self.is_locked() {
            return Err("never locked.".to_owned());
        }

        if unsafe { GlobalUnlock(self.h_global) }.is_err() && unsafe { GetLastError() }.is_err() {
            return Err("GlobalUnlock() failed.".to_owned());
        }

        self.locked = false;
        self.ptr = std::ptr::null_mut();

        Ok(())
    }

    pub fn free(&mut self) -> Result<(), String> {
        if !self.is_allocated() {
            return Err("never allocated.".to_owned());
        }

        match unsafe { GlobalFree(Some(self.h_global)) } {
            Ok(_) => {
                self.allocated = false;
                self.h_global = HGLOBAL::default();
                self.ptr = std::ptr::null_mut();
            }
            Err(e) => {
                return Err(format!("GlobalFree() failed: {}", e.to_string()));
            }
        };

        Ok(())
    }
}

impl Drop for GlobalMemory {
    fn drop(&mut self) {
        // avoid error
        if self.is_locked() {
            match self.unlock() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
            }
        }

        // avoid error
        if self.is_allocated() {
            match self.free() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
            }
        }
    }
}
