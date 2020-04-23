#![feature(const_in_array_repeat_expressions)]
#![feature(const_fn)]
#![feature(asm)]
#![no_std]
#![feature(let_chains)]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate embedded_hal;
extern crate f3;
extern crate heapless;

pub mod core {
    pub use cortex_m::{asm::wfi, register};
    pub use cortex_m_rt::{entry, exception, pre_init};
    pub use embedded_hal::{
        blocking::delay::{DelayMs, DelayUs},
        digital::v2::OutputPin,
        prelude::*,
        timer::CountDown,
    };
    pub use f3::hal::{
        flash::FlashExt,
        gpio::{
            gpioa::{PA5, PA6, PA7},
            gpiob::{PB6, PB7},
            gpioe::{PEx, PE3},
            Output, PushPull,
        },
        i2c::I2c,
        prelude::*,
        rcc::RccExt,
        rcc::{Clocks, APB1},
        spi::Spi,
        stm32f30x::{CorePeripherals, Peripherals as DevPeripherals, I2C1, SCB, SPI1, SYST, TIM2},
        time::Hertz,
        timer::Timer,
    };
}

pub mod tasks {
    pub trait Task: Sync {
        fn run(&mut self);
        fn get_stk_ptr(&self) -> *mut u8;
        fn update_stk_ptr(&self, p: *mut u8);
        fn get_state(&self) -> TaskState;
        fn set_state(&self, s: TaskState);
        fn get_prd(&self) -> u32;
    }

    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum TaskState {
        PreInit,
        Ready,
        Running,
        Suspended,
        Done,
    }

    pub mod accel;
    pub mod gyro;
    pub mod led;
}

pub mod task_man;
