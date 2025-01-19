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
    assignments: UnsafeCell<[Assignment; ASSIGNMENT_COUNT]>,
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
                let assignment = self.assignments.get().cast::<Assignment>().add(i);
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

        let mut prev_assignment_end = self.arena.get().cast::<u8>();

        // Iterate through each assignment record
        'record: for i in 0..ASSIGNMENT_COUNT {
            let next_assignment = self.assignments.get().cast::<Assignment>().add(i);
            let next_ptr = (*next_assignment).ptr as *mut u8;

            if next_ptr.is_null() {
                // If we find an empty assignment record, see if we can allocate there
                let align_offset = prev_assignment_end.align_offset(align);
                let effective_size = size + align_offset;

                // We need to find the next non-empty assignment record
                for j in i..ASSIGNMENT_COUNT {
                    let valid_assignment = self.assignments.get().cast::<Assignment>().add(j);
                    let valid_ptr = (*valid_assignment).ptr as *mut u8;

                    // This is the next non-empty assignment record
                    if !valid_ptr.is_null() {
                        let available = valid_ptr.byte_offset_from(prev_assignment_end) as usize;

                        // We can allocate here
                        if available >= effective_size {
                            let assignment = self.assignments.get().cast::<Assignment>().add(j);
                            let ptr = prev_assignment_end.add(align_offset);
                            *assignment = Assignment { ptr, size };
                            return ptr;
                        }

                        // TODO: A future improvement is i can skip ahead to j here

                        continue 'record;
                    }
                }

                // There are no more assignment records
                // We have the rest of the arena to allocate
                let available = self
                    .arena
                    .get()
                    .cast::<u8>()
                    .add(ARENA_SIZE)
                    .byte_offset_from(prev_assignment_end) as usize;

                // We can allocate here
                if available >= effective_size {
                    let assignment = self.assignments.get().cast::<Assignment>().add(i);
                    let ptr = prev_assignment_end.add(align_offset);
                    *assignment = Assignment { ptr, size };
                    return ptr;
                }

                // No room at the end of the arena
                return null_mut();
            } else {
                // If we find a non-empty assignment record, record its index and end address.
                prev_assignment_end = next_ptr.byte_add((*next_assignment).size);
            }
        }

        // We're outta records!
        return null_mut();
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        for i in 0..ASSIGNMENT_COUNT {
            let assignment = self.assignments.get().cast::<Assignment>().add(i);
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
    assignments: UnsafeCell::new(
        [Assignment {
            ptr: null(),
            size: 0,
        }; ASSIGNMENT_COUNT],
    ),
};
