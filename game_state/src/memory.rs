//! The allocated memory for the game state

use core::ffi::c_void;

#[cfg(target_os = "linux")]
extern "C" {
    pub(crate) fn mmap(
        addr: *const c_void,
        length: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: i64,
    ) -> *mut u8;
}

pub const MEMORY_BASE_ADDR: u64 = 0xcdcd_0000;
pub const MEMORY_LENGTH: usize = 2 * 1024 * 1024;

#[cfg(target_os = "linux")]
pub fn allocate_memory(base_addr: u64, length: usize) -> *mut u8 {
    const PROT_READ: i32 = 0x1;
    const PROT_WRITE: i32 = 0x2;
    const MAP_PRIVATE: i32 = 0x02;
    const MAP_ANON: i32 = 0x20;
    const MAP_FIXED: i32 = 0x10;
    const MAP_FAILED: isize = -1_isize;

    let res = unsafe {
        mmap(
            base_addr as *const c_void,
            length,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANON | MAP_FIXED,
            -1,
            0,
        )
    };

    assert!(res != MAP_FAILED as *mut u8);

    res
}

#[cfg(not(target_os = "linux"))]
compile_error!("Memory allocation not written for this operating system");

/// Memory chunk allocated for the game with a basic bump allocator
pub struct Memory {
    /// Has this memory been initialized by the game yet
    pub initialized: bool,

    /// Data bytes for this memory, allocated by the platform
    pub data: *mut u8,

    /// Size of the data allocation
    pub data_len: usize,

    /// Offset to the next allocation in the memory region
    pub next_allocation: usize,
}

impl Memory {
    /// Allocate a new chunk of memory
    #[cfg(target_os = "linux")]
    pub fn new(size: usize) -> Self {
        Self {
            initialized: false,
            data: allocate_memory(MEMORY_BASE_ADDR, MEMORY_LENGTH),
            data_len: size,
            next_allocation: 0,
        }
    }

    /// Allocate `T` in the allocated game memory
    ///
    /// # Panics
    ///
    /// * Out of allocated memory
    pub fn alloc<T: Sized>(&mut self) -> *mut T {
        assert!(
            self.next_allocation + std::mem::size_of::<T>() < self.data_len,
            "Out of game memory"
        );

        // Get the resulting address
        let result = unsafe { self.data.add(self.next_allocation) };

        // Bump the allocation to fit the requested type
        self.next_allocation += std::mem::size_of::<T>();

        // 64 bit align the next allocation
        self.next_allocation = (self.next_allocation + 0xf) & !0xf;

        // Return the pointer to the allocation
        result.cast::<T>()
    }

    /// Create a copy of the current data as a Vec<u8>
    pub fn data_as_vec(&self) -> Vec<u8> {
        unsafe { std::slice::from_raw_parts(self.data, self.data_len).to_vec() }
    }
}
