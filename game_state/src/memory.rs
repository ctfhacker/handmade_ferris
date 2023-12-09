//! The allocated memory for the game state

use core::ffi::c_void;
use core::marker::PhantomData;
use core::mem::size_of;
use core::sync::atomic::{AtomicBool, Ordering};

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

pub const MEMORY_BASE_ADDR: usize = 0xcdcd_0000;
pub const MEMORY_LENGTH: usize = 2 * 1024 * 1024;

static ALLOCATED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "linux")]
pub fn allocate_memory() -> *mut u8 {
    const PROT_READ: i32 = 0x1;
    const PROT_WRITE: i32 = 0x2;
    const MAP_PRIVATE: i32 = 0x02;
    const MAP_ANON: i32 = 0x20;
    const MAP_FIXED: i32 = 0x10;
    const MAP_FAILED: isize = -1_isize;

    assert!(
        !ALLOCATED.load(Ordering::SeqCst),
        "Attempted to allocate game memory twice"
    );

    // Allocate the memory at the requested base address
    let res = unsafe {
        mmap(
            MEMORY_BASE_ADDR as *const c_void,
            MEMORY_LENGTH,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANON | MAP_FIXED,
            -1,
            0,
        )
    };

    assert!(res != MAP_FAILED as *mut u8);

    // Globally signal that the game memory has been allocated
    ALLOCATED.store(true, Ordering::SeqCst);

    res
}

#[cfg(not(target_os = "linux"))]
compile_error!("Memory allocation not written for this operating system");

/// Memory chunk allocated for the game with a basic bump allocator
pub struct Memory {
    /// Has this memory been initialized by the game yet
    pub initialized: bool,

    /// Offset to the next allocation in the memory region
    pub next_allocation: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct Allocation<T> {
    /// Index into the memory for this allocation
    index: usize,

    /// The type of this allocation
    phantom: PhantomData<T>,
}

impl<T> core::default::Default for Allocation<T> {
    fn default() -> Self {
        Self {
            index: !0,
            phantom: PhantomData,
        }
    }
}

impl<T> core::ops::Deref for Allocation<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The only way to create an Allocation is through `alloc` which checks
        //         that the index was in bounds
        unsafe { &*((MEMORY_BASE_ADDR + self.index) as *const T) }
    }
}

impl<T> core::ops::DerefMut for Allocation<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The only way to create an Allocation is through `alloc` which checks
        //         that the index was in bounds
        unsafe { &mut *((MEMORY_BASE_ADDR + self.index) as *mut T) }
    }
}

impl Memory {
    /// Allocate a new chunk of memory
    #[cfg(target_os = "linux")]
    pub fn new() -> Self {
        // Allocate the memory for the game
        allocate_memory();

        //
        Self {
            initialized: false,
            next_allocation: 0,
        }
    }

    /// Allocate `T` in the allocated game memory
    ///
    /// # Panics
    ///
    /// * Out of allocated memory
    pub fn alloc<T: Sized>(&mut self) -> Allocation<T> {
        let size = size_of::<T>();

        // SAFETY: This is the main safety check to ensure all allocations are in bounds
        assert!(
            self.next_allocation + size < MEMORY_LENGTH,
            "Out of game memory"
        );

        // Get the index for this allocation
        let index = self.next_allocation;

        // Bump the allocation to fit the requested type
        self.next_allocation += size;

        // 64 bit align the next allocation
        self.next_allocation = (self.next_allocation + 0xf) & !0xf;

        Allocation {
            index,
            phantom: PhantomData,
        }
    }

    /// Create a copy of the current data as a Vec<u8>
    pub fn data_as_vec(&self) -> Vec<u8> {
        unsafe { std::slice::from_raw_parts(MEMORY_BASE_ADDR as *const u8, MEMORY_LENGTH).to_vec() }
    }
}
