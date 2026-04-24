use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

/// A simple bump allocator for the DOS environment.
///
/// This allocator does not support deallocation, but it is sufficient for
/// the needs of rustid in a DOS environment where allocations are
/// relatively few and live for the duration of the program.
pub struct DosAllocator {
    start: AtomicUsize,
    end: AtomicUsize,
}

impl DosAllocator {
    /// Creates a new, uninitialized allocator.
    pub const fn new() -> Self {
        Self {
            start: AtomicUsize::new(0),
            end: AtomicUsize::new(0),
        }
    }

    /// Initializes the allocator with a memory range.
    ///
    /// # Safety
    /// This function must be called only once and with a valid memory range.
    pub unsafe fn init(&self, start: usize, size: usize) {
        self.start.store(start, Ordering::SeqCst);
        self.end.store(start + size, Ordering::SeqCst);
    }
}

unsafe impl GlobalAlloc for DosAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        loop {
            let current_start = self.start.load(Ordering::Relaxed);
            let current_end = self.end.load(Ordering::Relaxed);

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

            if self
                .start
                .compare_exchange_weak(current_start, alloc_end, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return alloc_start as *mut u8;
            }
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
    // We have at least 64KB allocated via min_alloc in the EXE header.
    // The _heap symbol marks the beginning of this space.
    let heap_start = &raw mut _heap as usize;
    // 64KB is a safe default for DOS rustid.
    unsafe { ALLOCATOR.init(heap_start, 64 * 1024) };
}
