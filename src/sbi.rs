use core::arch::asm;

pub fn ecall(
    arg0: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    fid: u64,
    eid: u64,
) -> Result<u64, u64> {
    let mut a0 = arg0;
    let mut a1 = arg1;

    unsafe {
        asm!("ecall", inlateout("a0") a0, inlateout("a1") a1, in("a2") arg2, in("a3") arg3, in("a4") arg4, in("a5") arg5, in("a6") fid, in("a7") eid);

        if a0 == 0 {
            return Ok(a1);
        }

        Err(a0)
    }
}
