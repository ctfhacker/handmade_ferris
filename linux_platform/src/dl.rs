//! Simple wrapper for symbol resolution using dlopen/dlsym

use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_void};
use std::marker::PhantomData;

use game_state::{Game, State};

#[link(name="dl")]
extern "C" {
    pub(crate) fn dlopen(filename: *const c_char, flags: u32) -> Handle;
    pub(crate) fn dlclose(handle: Handle);
    pub(crate) fn dlsym(handle: Handle, symbol: *const c_char) -> *mut c_void;
    pub(crate) fn dlerror() -> *const c_char;
}

/// Lazy funcdtion call binding
pub const RTLD_LAZY: u32 = 1;

/// Handle to an opened shared library
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Handle(pub usize);

/// Dynamically loaded functions from the game
pub struct GameFuncs {
    /// Handle to the library where the symbols are loaded from. This is kept around to
    /// enable `Drop`
    pub handle: Handle,

    /// Dummy test function
    pub game_update_and_render: Symbol<extern fn(&Game, &mut State)>,
}

impl GameFuncs {
    /// Drop the old game library and reload the new one
    pub fn reload(self) -> Self {
        // Drop the old library handle
        drop(self);

        // Reload the new game handle
        get_game_funcs()
    }
}


impl Drop for GameFuncs {
    fn drop(&mut self) {
        unsafe { 
            dlclose(self.handle);
        }
    }
}

/// A function loaded via `dlsym`
pub struct Symbol<T> {
    /// Handle to the opened symbol
    handle: *mut c_void,

    /// Type of handle for this function
    phantom: PhantomData<T>
}

impl<T> std::ops::Deref for Symbol<T> {
    type Target = T;

    fn deref(&self) -> &T {
        #[allow(clippy::ptr_as_ptr)]
        unsafe {
            &*(&self.handle as *const *mut _ as *const T)
        }
    }
}

/// Get the requested [`Symbol`] by export name using the given library handle
pub fn get_symbol<T>(library: Handle, symbol_name: &str) -> Result<Symbol<T>, CString> {
    // Get the `game_update_and_render` func from the game library
    unsafe {
        let symbol_name = CString::new(symbol_name)
            .unwrap_or_else(|_| panic!("CString failed for {}", symbol_name));

        let handle = dlsym(library, symbol_name.as_ptr().cast::<i8>());

        // If `dlsym` failed, return the error message from `dlerror`
        if handle.is_null() {
            return Err(CStr::from_ptr(dlerror()).into());
        }

        // Return the found symbol
        Ok(Symbol {
            handle,
            phantom: PhantomData
        })
    }

}

/// Location of the copied game logic library used to enable hot reload
const TMP_FILE: &str = "/tmp/.libgame.so";

/// Load and return the function pointers from the game code
pub fn get_game_funcs() -> GameFuncs {
    // Copy the current game library into a temp file for hot reload. Ignore the failure
    // copy case and pick up the game logic on the next frame
    let _discard = std::fs::copy("./target/release/libgame.so", TMP_FILE);

    // Get the temporary library file
    let library = CString::new(TMP_FILE).expect("CString failed for tmp library");

    unsafe {
        // Open the  current game dynamic library
        let handle = dlopen(library.as_ptr(), RTLD_LAZY);
        assert!(handle.0 != 0, "libgame.so not found");

        // Get the `game_update_and_render` export
        let game_update_and_render = get_symbol(handle, "game_update_and_render")
            .unwrap();

        // Return the exported game functions
        GameFuncs {
            handle,
            game_update_and_render
        }
    }
}
