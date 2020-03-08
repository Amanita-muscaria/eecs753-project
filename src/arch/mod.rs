pub mod cortex_m4f;

pub trait Arch {
    fn wfi(&self);
    fn register_systick_cb(&mut self, cb: fn());
    fn set_systick(&self, ticks: usize);
}
