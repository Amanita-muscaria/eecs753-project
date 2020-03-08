use rtos::arch::arch_none::ArchNone;
use rtos::kernel::*;
use rtos::task::*;

#[test]
fn periodic_task_integration() {
    let t1 = Prd { prd: 32 };
    let t2 = Prd2 { prd: 64 };
    let t3 = Prd { prd: 128 };
    let t4 = Prd { prd: 256 };
    let mut k = RMS::new(ArchNone {});
}
