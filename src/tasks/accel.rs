use crate::core::{I2c, I2C1, PB6, PB7};
use crate::tasks::{task_done, Task, TaskState};
use core::cell::Cell;
use core::mem::transmute;
use f3::{
    hal::gpio::AF4,
    lsm303dlhc::{AccelOdr, I16x3, Sensitivity},
    Lsm303dlhc,
};

const STK_SIZE: usize = 512;
const PERIOD: u32 = 5;
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
            .set((&mut ACCEL_STACK[STK_SIZE - 10]) as *mut u32);

        self.stk_ptr
            .get()
            .write(transmute::<*mut AccelTask, u32>(self));
        self.stk_ptr.get().offset(6).write(AccelTask::run as u32);
        self.stk_ptr.get().offset(7).write(0x21000000);
        self.stk_ptr.set(self.stk_ptr.get().sub(8));
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
            let a = self.accel.as_mut().unwrap();
            self.buff[self.buff_head] = a.accel().unwrap();
            self.buff_head = if self.buff_head + 1 >= BUFF_CAP {
                0
            } else {
                self.buff_head + 1
            };
            task_done(1);
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
