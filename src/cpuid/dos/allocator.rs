use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::null_mut;

/// A simple bump allocator for the DOS environment.
///
/// This allocator does not support deallocation, but it is sufficient for
/// the needs of rustid in a DOS environment where allocations are
/// relatively few and live for the duration of the program.
///
/// Since DOS is a single-threaded environment, we use non-atomic operations
/// to maintain compatibility with 386 processors which lack CMPXCHG.
pub struct DosAllocator {
    start: UnsafeCell<usize>,
    end: UnsafeCell<usize>,
}

// SAFETY: DOS is a single-threaded/single-tasking environment.
// Our allocator is only accessed by the main program and we don't
// use interrupts for allocation.
unsafe impl Sync for DosAllocator {}

impl DosAllocator {
    /// Creates a new, uninitialized allocator.
    pub const fn new() -> Self {
        Self {
            start: UnsafeCell::new(0),
            end: UnsafeCell::new(0),
        }
    }

    /// Initializes the allocator with a memory range.
    ///
    /// # Safety
    /// This function must be called only once and with a valid memory range.
    pub unsafe fn init(&self, start: usize, size: usize) {
        unsafe {
            *self.start.get() = start;
            *self.end.get() = start + size;
        }
    }
}

unsafe impl GlobalAlloc for DosAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        unsafe {
            let current_start = *self.start.get();
            let current_end = *self.end.get();

            if current_start == 0 {
                return null_mut();
            }

            let alloc_start = (current_start + align - 1) & !(align - 1);
            let alloc_end = match alloc_start.checked_add(size) {
                Some(end) => end,
                None => return null_mut(),
            };

            if alloc_end > current_end {
                return null_mut();
            }

            *self.start.get() = alloc_end;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator does not deallocate.
    }
}

#[global_allocator]
static ALLOCATOR: DosAllocator = DosAllocator::new();

unsafe extern "C" {
    static mut _heap: u8;
}

/// Initializes the global allocator for DOS.
///
/// # Safety
/// This function must be called early in the program's execution,
/// before any allocations occur.
pub unsafe fn init_heap() {
    let heap_start = &raw mut _heap as usize;

    // Use a safe default for Real Mode (within the current 64KB segment)
    let heap_size = 0x10000usize.saturating_sub(heap_start & 0xFFFF);

    unsafe { ALLOCATOR.init(heap_start, heap_size) };
}
