use crate::core::{I2c, I2C1, PB6, PB7};
use crate::tasks::{Task, TaskState};
use f3::{
    hal::gpio::AF4,
    lsm303dlhc::{AccelOdr, I16x3, Sensitivity},
    Lsm303dlhc,
};

const STK_SIZE: usize = 512;
const PERIOD: usize = 0;
const STACK: [u8; STK_SIZE] = [0; STK_SIZE];
const BUFF_CAP: usize = 16;

type AccelI2c = I2c<I2C1, (PB6<AF4>, PB7<AF4>)>;

pub struct AccelTask {
    state: TaskState,
    stk_ptr: *mut u8,
    prd: usize,
    accel: Option<Lsm303dlhc>,
    buff: [I16x3; BUFF_CAP],
    buff_head: usize,
}

impl AccelTask {
    pub fn init(&mut self, i: AccelI2c) {
        let mut accel = Lsm303dlhc::new(i).unwrap();
        accel.accel_odr(AccelOdr::Hz10).unwrap();
        accel.set_accel_sensitivity(Sensitivity::G1).unwrap();
        self.accel = Some(accel);
    }

    pub const fn default() -> Self {
        AccelTask {
            state: TaskState::Ready,
            stk_ptr: STACK.as_ptr() as *mut u8,
            prd: PERIOD,
            accel: None,
            buff: [I16x3 { x: 0, y: 0, z: 0 }; BUFF_CAP],
            buff_head: 0,
        }
    }
}

impl Task for AccelTask {
    fn run(&mut self) {
        let a = self.accel.as_mut().unwrap();
        self.buff[self.buff_head] = a.accel().unwrap();
        self.buff_head = if self.buff_head + 1 >= BUFF_CAP {
            0
        } else {
            self.buff_head + 1
        };
    }

    fn get_stk_ptr(&self) -> *mut u8 {
        self.stk_ptr
    }

    fn update_stk_ptr(&mut self, p: *mut u8) {
        self.stk_ptr = p;
    }

    fn get_state(&self) -> TaskState {
        self.state
    }

    fn set_state(&mut self, s: TaskState) {
        self.state = s;
    }

    fn get_prd(&self) -> usize {
        self.prd
    }
}
