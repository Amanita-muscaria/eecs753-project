#![no_std]
#![allow(dead_code)]
#![feature(const_fn)]

extern crate cortex_m;
extern crate cortex_m_rt;
pub extern crate cortex_m_rt as rt;
extern crate stm32f3;
extern crate stm32f3 as stm32;

use core::sync::atomic::{AtomicBool, Ordering};

pub mod arch;
pub mod kernel;
pub mod task;
pub mod task_control;
