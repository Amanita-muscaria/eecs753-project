use crate::core::{Output, OutputPin, PushPull, Spi, PA5, PA6, PA7, PE3, SPI1};
use crate::tasks::{Task, TaskState};
use f3::{hal::gpio::AF5, l3gd20::I16x3, L3gd20};

const STK_SIZE: usize = 512;
const PERIOD: usize = 0;
const STACK: [u8; STK_SIZE] = [0; STK_SIZE];
const BUFF_CAP: usize = 8;

type GyroSpi = Spi<SPI1, (PA5<AF5>, PA6<AF5>, PA7<AF5>)>;

pub struct GyroTask {
    gyro: Option<L3gd20>,
    buff: [I16x3; BUFF_CAP],
    buff_head: usize,
    state: TaskState,
    stk_ptr: *mut u8,
    prd: usize,
}

impl GyroTask {
    pub fn init(&mut self, s: GyroSpi, mut cs: PE3<Output<PushPull>>) {
        cs.set_high().unwrap();
        self.gyro = Some(L3gd20::new(s, cs).unwrap());
        self.state = TaskState::Ready;
    }

    pub const fn default() -> Self {
        Self {
            gyro: None,
            buff: [I16x3 { x: 0, y: 0, z: 0 }; BUFF_CAP],
            buff_head: 0,
            state: TaskState::PreInit,
            stk_ptr: STACK.as_ptr() as *mut u8,
            prd: PERIOD,
        }
    }
}

impl Task for GyroTask {
    fn run(&mut self) {
        let g = self.gyro.as_mut().unwrap();
        self.buff[self.buff_head] = g.gyro().unwrap();
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
