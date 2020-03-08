use crate::arch::Arch;
use cortex_m::asm::wfi;
// use cortex_m::peripheral::{syst::RegisterBlock, SYST};
use rt::exception;
use stm32f3::stm32f303::SYST;

pub static CORTEXM4F: CortexM4F = CortexM4F::new();

pub struct CortexM4F {
    sys_cb: Option<fn()>,
}

impl CortexM4F {
    const fn new() -> Self {
        Self { sys_cb: None }
    }

    fn handle_systick(&self) {
        if let Some(c) = self.sys_cb {
            c();
        }
    }
}

impl Arch for CortexM4F {
    fn wfi(&self) {
        wfi();
    }

    fn register_systick_cb(&mut self, cb: fn()) {
        self.sys_cb = Some(cb);
    }

    fn set_systick(&self, ticks: usize) {
        unimplemented!()
    }
}

#[allow(non_snake_case)]
#[exception]
fn SysTick() {
    CORTEXM4F.handle_systick();
}
