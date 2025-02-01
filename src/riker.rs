// I named the allocator Riker, because he seems like he'd handle that sort of administrative stuff

use core::{
    alloc::GlobalAlloc,
    cell::UnsafeCell,
    ptr::{null, null_mut},
};

// The heap size we're setting aside, in Bytes
const ARENA_SIZE: usize = 128 * 1024;

// The maximum supported align, in Bytes
const MAX_SUPPORTED_ALIGN: usize = 4096;

// The maximum number of assignment records
// Total size can be calculated as ASSIGNMENTS_COUNT * size_of::<Assignment>()
const ASSIGNMENT_COUNT: usize = 512;

// A very basic allocator
pub struct Riker {
    arena: UnsafeCell<[u8; ARENA_SIZE]>,
    records: UnsafeCell<[Assignment; ASSIGNMENT_COUNT]>,
}

// A record of a memory assignment
#[derive(Clone, Copy)]
struct Assignment {
    ptr: *const u8,
    size: usize,
}

unsafe impl Sync for Riker {}

impl Riker {
    // Calculates the heap space available for allocation
    pub fn remaining(&self) -> usize {
        let mut prev_assignment_ptr = self.arena.get().cast::<u8>();
        let mut prev_assignment_size = 0 as usize;

        for i in 0..ASSIGNMENT_COUNT {
            unsafe {
                let assignment = self.records.get().cast::<Assignment>().add(i);
                let next_ptr = (*assignment).ptr as *mut u8;

                if !next_ptr.is_null() {
                    prev_assignment_ptr = next_ptr;
                    prev_assignment_size = (*assignment).size;
                }
            }
        }

        unsafe {
            self.arena
                .get()
                .cast::<u8>()
                .add(ARENA_SIZE)
                .byte_offset_from(prev_assignment_ptr.byte_add(prev_assignment_size))
                as usize
        }
    }
}

unsafe impl GlobalAlloc for Riker {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        if align > MAX_SUPPORTED_ALIGN {
            return null_mut();
        }

        let mut prev_record_end = self.arena.get().cast::<u8>();

        // Iterate through each assignment record
        let mut next_idx = 0;
        'outer: while next_idx < ASSIGNMENT_COUNT {
            let next_record = self.records.get().cast::<Assignment>().add(next_idx);
            let next_ptr = (*next_record).ptr as *mut u8;

            if next_ptr.is_null() {
                // If we find an empty assignment record, see if we can allocate there
                let align_offset = prev_record_end.align_offset(align);
                let effective_size = size + align_offset;

                // We need to find the next non-empty assignment record
                for valid_idx in next_idx..ASSIGNMENT_COUNT {
                    let valid_record = self.records.get().cast::<Assignment>().add(valid_idx);
                    let valid_ptr = (*valid_record).ptr as *mut u8;

                    // This is the next non-empty assignment record
                    if !valid_ptr.is_null() {
                        let available = valid_ptr.byte_offset_from(prev_record_end) as usize;

                        // We can allocate here
                        if available >= effective_size {
                            let new_record = self.records.get().cast::<Assignment>().add(valid_idx);
                            let ptr = prev_record_end.add(align_offset);
                            *new_record = Assignment { ptr, size };
                            return ptr;
                        }

                        next_idx = valid_idx;
                        continue 'outer;
                    }
                }

                // There are no more assignment records
                // We have the rest of the arena to allocate
                let available = self
                    .arena
                    .get()
                    .cast::<u8>()
                    .add(ARENA_SIZE)
                    .byte_offset_from(prev_record_end) as usize;

                // We can allocate here
                if available >= effective_size {
                    let new_record = self.records.get().cast::<Assignment>().add(next_idx);
                    let ptr = prev_record_end.add(align_offset);
                    *new_record = Assignment { ptr, size };
                    return ptr;
                }

                // No room at the end of the arena
                return null_mut();
            } else {
                // If we find a non-empty assignment record, record its index and end address.
                prev_record_end = next_ptr.byte_add((*next_record).size);
                next_idx += 1;
            }
        }

        // We're outta records!
        return null_mut();
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        for i in 0..ASSIGNMENT_COUNT {
            let assignment = self.records.get().cast::<Assignment>().add(i);
            let next_ptr = (*assignment).ptr as *mut u8;

            if next_ptr == ptr {
                // Clear the assignment record
                (*assignment).ptr = null();
                return;
            }
        }
    }
}

#[global_allocator]
pub static ALLOC: Riker = Riker {
    arena: UnsafeCell::new([0x55; ARENA_SIZE]),
    records: UnsafeCell::new(
        [Assignment {
            ptr: null(),
            size: 0,
        }; ASSIGNMENT_COUNT],
    ),
};
