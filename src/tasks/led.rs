use crate::core::{CountDown, Hertz, Output, OutputPin, PEx, PushPull, Timer, TIM2};
use crate::tasks::{task_done, Task, TaskState};
use core::cell::Cell;
use core::mem::transmute;

const STK_SIZE: usize = 512;
const PERIOD: u32 = 13;
static mut LED_STACK: [u32; STK_SIZE] = [0; STK_SIZE];

type LedPin = PEx<Output<PushPull>>;

pub struct LedTask {
    state: Cell<TaskState>,
    stk_ptr: Cell<*mut u32>,
    prd: u32,
    leds: Option<[Led; 8]>,
    d: Option<Timer<TIM2>>,
}

impl LedTask {
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn init(
        &mut self,
        p0: LedPin,
        p1: LedPin,
        p2: LedPin,
        p3: LedPin,
        p4: LedPin,
        p5: LedPin,
        p6: LedPin,
        p7: LedPin,
        d: Timer<TIM2>,
    ) {
        let l = [
            p0.into(),
            p1.into(),
            p2.into(),
            p3.into(),
            p4.into(),
            p5.into(),
            p6.into(),
            p7.into(),
        ];
        self.leds = Some(l);
        self.d = Some(d);
        self.stk_ptr
            .set((&mut LED_STACK[STK_SIZE - 10]) as *mut u32);

        self.stk_ptr
            .get()
            .write(transmute::<*mut LedTask, u32>(self));
        self.stk_ptr.get().offset(6).write(LedTask::run as u32);
        self.stk_ptr.get().offset(7).write(0x21000000);
        self.stk_ptr.set(self.stk_ptr.get().sub(8));
        self.state.set(TaskState::Ready);
    }

    pub const fn default() -> Self {
        Self {
            state: Cell::new(TaskState::PreInit),
            stk_ptr: Cell::new(0 as *mut u32),
            prd: PERIOD,
            leds: None,
            d: None,
        }
    }
}

impl Task for LedTask {
    fn run(&mut self) {
        loop {
            let leds = self.leds.as_mut().unwrap();
            let mut d = self.d.as_mut().unwrap();
            for l in leds.iter_mut() {
                l.toggle();
                wait(&mut d, 1);
            }
            task_done(2);
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

pub struct Led {
    pex: LedPin,
    on: bool,
}

impl Led {
    pub fn off(&mut self) {
        self.pex.set_low().unwrap();
    }

    pub fn on(&mut self) {
        self.pex.set_high().unwrap();
    }

    pub fn toggle(&mut self) {
        if self.on {
            self.off();
            self.on = false;
        } else {
            self.on();
            self.on = true;
        }
    }
}

impl From<LedPin> for Led {
    fn from(pex: LedPin) -> Self {
        Led { pex, on: false }
    }
}

fn wait(d: &mut Timer<TIM2>, timeout: u32) {
    d.start(Hertz(timeout));
    while d.wait().is_err() {}
}

unsafe impl Sync for LedTask {}
