use crate::core::{Output, OutputPin, PushPull, Spi, PA5, PA6, PA7, PE3, SPI1};
use crate::tasks::{task_done, Task, TaskState};
use core::cell::Cell;
use core::mem::transmute;
use f3::{hal::gpio::AF5, l3gd20::I16x3, L3gd20};

const STK_SIZE: usize = 512;
const PERIOD: u32 = 7;
static mut GYRO_STACK: [u32; STK_SIZE] = [0; STK_SIZE];
const BUFF_CAP: usize = 8;

type GyroSpi = Spi<SPI1, (PA5<AF5>, PA6<AF5>, PA7<AF5>)>;

pub struct GyroTask {
    gyro: Option<L3gd20>,
    buff: [I16x3; BUFF_CAP],
    buff_head: usize,
    state: Cell<TaskState>,
    stk_ptr: Cell<*mut u32>,
    prd: u32,
}

impl GyroTask {
    pub unsafe fn init(&mut self, s: GyroSpi, mut cs: PE3<Output<PushPull>>) {
        cs.set_high().unwrap();
        self.gyro = Some(L3gd20::new(s, cs).unwrap());
        self.state.set(TaskState::Ready);
        self.stk_ptr
            .set((&mut GYRO_STACK[STK_SIZE - 10]) as *mut u32);

        self.stk_ptr
            .get()
            .write(transmute::<*mut GyroTask, u32>(self));
        self.stk_ptr.get().offset(6).write(GyroTask::run as u32);
        self.stk_ptr.get().offset(7).write(0x21000000);
        self.stk_ptr.set(self.stk_ptr.get().sub(8));
    }

    pub const fn default() -> Self {
        Self {
            gyro: None,
            buff: [I16x3 { x: 0, y: 0, z: 0 }; BUFF_CAP],
            buff_head: 0,
            state: Cell::new(TaskState::PreInit),
            stk_ptr: Cell::new(0 as *mut u32),
            prd: PERIOD,
        }
    }
}

impl Task for GyroTask {
    fn run(&mut self) {
        loop {
            let g = self.gyro.as_mut().unwrap();
            self.buff[self.buff_head] = g.gyro().unwrap();
            self.buff_head = if self.buff_head + 1 >= BUFF_CAP {
                0
            } else {
                self.buff_head + 1
            };
            task_done(0);
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

unsafe impl Sync for GyroTask {}
