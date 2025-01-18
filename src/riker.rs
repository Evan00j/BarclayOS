// This is all just ripped from https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html
// I named the allocator Riker, because he seems like he'd handle that sort of administrative stuff

use core::{
    alloc::GlobalAlloc,
    cell::UnsafeCell,
    ptr::null_mut,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
};

const ARENA_SIZE: usize = 128 * 1024;
const MAX_SUPPORTED_ALIGN: usize = 4096;

pub struct Riker {
    arena: UnsafeCell<[u8; ARENA_SIZE]>,
    pub remaining: AtomicUsize, // we allocate from the top, counting down
}

unsafe impl Sync for Riker {}

unsafe impl GlobalAlloc for Riker {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        // `Layout` contract forbids making a `Layout` with align=0, or align not power of 2.
        // So we can safely use a mask to ensure alignment without worrying about UB.
        let align_mask_to_round_down = !(align - 1);

        if align > MAX_SUPPORTED_ALIGN {
            return null_mut();
        }

        let mut allocated = 0;

        if self
            .remaining
            .fetch_update(Relaxed, Relaxed, |mut remaining| {
                if size > remaining {
                    return None;
                }

                remaining -= size;
                remaining &= align_mask_to_round_down;
                allocated = remaining;

                Some(remaining)
            })
            .is_err()
        {
            return null_mut();
        };

        self.arena.get().cast::<u8>().add(allocated)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        // TODO, but not todo!
        // Don't want a panic yet
    }
}

#[global_allocator]
pub static ALLOC: Riker = Riker {
    arena: UnsafeCell::new([0x55; ARENA_SIZE]),
    remaining: AtomicUsize::new(ARENA_SIZE),
};
