#![feature(const_in_array_repeat_expressions)]
#![feature(const_fn)]
#![feature(asm)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate embedded_hal;
extern crate f3;
extern crate stm32f3;

pub mod core {
    pub use cortex_m::{
        asm,
        peripheral::{nvic, NVIC},
        register,
    };
    pub use cortex_m_rt::{entry, exception, pre_init};
    pub use embedded_hal::{
        blocking::delay::{DelayMs, DelayUs},
        digital::v2::OutputPin,
    };
    pub use f3::hal::{
        delay::Delay,
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
        spi::Spi,
        stm32f30x,
        stm32f30x::{Interrupt, I2C1, SPI1},
        time::U32Ext,
    };
    pub mod stm_p {
        pub use stm32f3::stm32f303::*;
    }
}

pub mod tasks {
    pub trait Task {
        fn run(&mut self);
        fn get_stk_ptr(&self) -> *mut u8;
        fn update_stk_ptr(&mut self, p: *mut u8);
        fn get_state(&self) -> TaskState;
        fn set_state(&mut self, s: TaskState);
        fn get_prd(&self) -> usize;
    }

    #[derive(Copy, Clone)]
    pub enum TaskState {
        Ready,
        Running,
        Suspended,
    }

    pub mod accel;
    pub mod gyro;
    pub mod led;
}
