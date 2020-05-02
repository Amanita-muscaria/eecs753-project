use crate::core::{wfe, I2c, I2C1, PB6, PB7, SCB};
use crate::tasks::{Task, TaskState};
use core::cell::Cell;
use core::mem::transmute;
use f3::{
    hal::gpio::AF4,
    lsm303dlhc::{AccelOdr, I16x3, Sensitivity},
    Lsm303dlhc,
};

const STK_SIZE: usize = 512;
const PERIOD: u32 = 11;
static mut ACCEL_STACK: [u32; STK_SIZE] = [0; STK_SIZE];
const BUFF_CAP: usize = 16;

type AccelI2c = I2c<I2C1, (PB6<AF4>, PB7<AF4>)>;

pub struct AccelTask {
    state: Cell<TaskState>,
    stk_ptr: Cell<*mut u32>,
    prd: u32,
    accel: Option<Lsm303dlhc>,
    buff: [I16x3; BUFF_CAP],
    buff_head: usize,
}

impl AccelTask {
    pub unsafe fn init(&mut self, i: AccelI2c) {
        let mut accel = Lsm303dlhc::new(i).unwrap();
        accel.accel_odr(AccelOdr::Hz10).unwrap();
        accel.set_accel_sensitivity(Sensitivity::G1).unwrap();
        self.accel = Some(accel);
        self.state = Cell::new(TaskState::Ready);
        self.stk_ptr
            .set(ACCEL_STACK.as_ptr().offset((STK_SIZE - 16) as isize) as *mut u32);
        *(self.stk_ptr.get()) = transmute::<*mut AccelTask, u32>(self);
    }

    pub const fn default() -> Self {
        AccelTask {
            state: Cell::new(TaskState::PreInit),
            stk_ptr: Cell::new(0 as *mut u32),
            prd: PERIOD,
            accel: None,
            buff: [I16x3 { x: 0, y: 0, z: 0 }; BUFF_CAP],
            buff_head: 0,
        }
    }
}

impl Task for AccelTask {
    fn run(&mut self) {
        loop {
            self.state.set(TaskState::Running);
            let a = self.accel.as_mut().unwrap();
            self.buff[self.buff_head] = a.accel().unwrap();
            self.buff_head = if self.buff_head + 1 >= BUFF_CAP {
                0
            } else {
                self.buff_head + 1
            };
            self.state.set(TaskState::Done);
            SCB::set_pendsv();
        }
    }

    fn get_stk_ptr(&self) -> *mut u32 {
        self.stk_ptr.get()
    }

    fn update_stk_ptr(&self, p: *mut u32) {
        self.stk_ptr.set(p);
    }

    fn get_state(&self) -> TaskState {
        self.state.get()
    }

    fn set_state(&self, s: TaskState) {
        self.state.set(s);
    }

    fn get_prd(&self) -> u32 {
        self.prd
    }
}

unsafe impl Sync for AccelTask {}
