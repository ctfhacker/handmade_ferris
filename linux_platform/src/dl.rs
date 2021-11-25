//! Simple wrapper for symbol resolution using dlopen/dlsym

use std::ffi::CString;
use std::os::raw::c_char;

#[link(name="dl")]
extern "C" {
    pub(crate) fn dlopen(filename: *const c_char, flags: u32) -> Handle;
    pub(crate) fn dlclose(handle: Handle);
    pub(crate) fn dlsym(handle: Handle, symbol: *const c_char) -> extern fn() -> u64;
}

/// Lazy funcdtion call binding
pub const RTLD_LAZY: u32 = 1;

/// Handle to an opened shared library
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Handle(pub usize);

/// Dynamically loaded functions from the game
pub struct GameFuncs {
    pub handle: Handle,

    /// Dummy test function
    pub callme: extern fn() -> u64
}


impl Drop for GameFuncs {
    fn drop(&mut self) {
        unsafe { 
            dlclose(self.handle);
        }
    }
}

/// Load and return the function pointers from the game code
pub fn get_game_funcs() -> GameFuncs {
    const TMP_FILE: &'static str = "/tmp/.libgame.so";
    std::fs::copy("./libgame.so", TMP_FILE).expect("Failed to copy libgame.so");

    let library = CString::new(TMP_FILE).expect("CString failed for /tmp/libgame.so");
    let callme_sym = CString::new("callme").expect("CString failed for callme");

    unsafe {
        let handle = dlopen(library.as_ptr(), RTLD_LAZY);
        assert!(handle.0 != 0, "libgame.so not found");

        let callme   = dlsym(handle, callme_sym.as_ptr());

        GameFuncs {
            handle,
            callme
        }
    }
}
