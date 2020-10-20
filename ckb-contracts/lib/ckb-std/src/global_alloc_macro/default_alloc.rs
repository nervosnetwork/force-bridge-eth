/// Defines global allocator
///
///
/// # Example
///
/// ```
/// // define global allocator
/// default_alloc!()
///
/// // Default allocator uses a mixed allocation strategy:
/// //
/// // * Fixed block heap, only allocate fixed size(64B) memory block
/// // * Dynamic memory heap, allocate any size memory block
/// //
/// // User can invoke macro with arguments to customize the heap size
/// // The default heap size arguments are:
/// // (fixed heap size 4KB, dynamic heap size 516KB, dynamic heap min memory block 64B)
/// default_alloc!(4 * 1024, 516 * 1024, 64)
/// ```
#[macro_export]
macro_rules! default_alloc {
    () => {
        default_alloc!(4 * 1024, 516 * 1024, 64);
    };
    ($fixed_block_heap_size:expr, $heap_size:expr, $min_block_size:expr) => {
        static mut _BUDDY_HEAP: [u8; $heap_size] = [0u8; $heap_size];
        static mut _FIXED_BLOCK_HEAP: [u8; $fixed_block_heap_size] = [0u8; $fixed_block_heap_size];

        #[global_allocator]
        static ALLOC: $crate::buddy_alloc::NonThreadsafeAlloc = unsafe {
            let fast_param = $crate::buddy_alloc::FastAllocParam::new(
                _FIXED_BLOCK_HEAP.as_ptr(),
                $fixed_block_heap_size,
            );
            let buddy_param = $crate::buddy_alloc::BuddyAllocParam::new(
                _BUDDY_HEAP.as_ptr(),
                $heap_size,
                $min_block_size,
            );
            $crate::buddy_alloc::NonThreadsafeAlloc::new(fast_param, buddy_param)
        };
    };
}
