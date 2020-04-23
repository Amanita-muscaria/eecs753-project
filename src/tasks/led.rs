use crate::core::{CountDown, Hertz, Output, OutputPin, PEx, PushPull, Timer, TIM2};
use crate::tasks::{Task, TaskState};
use core::cell::Cell;

const STK_SIZE: usize = 512;
const PERIOD: u32 = 21;
const STACK: [u8; STK_SIZE] = [0; STK_SIZE];

type LedPin = PEx<Output<PushPull>>;

pub struct LedTask {
    state: Cell<TaskState>,
    stk_ptr: Cell<*mut u8>,
    prd: u32,
    leds: Option<[Led; 8]>,
    d: Option<Timer<TIM2>>,
}

impl LedTask {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
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
    }

    pub const fn default() -> Self {
        Self {
            state: Cell::new(TaskState::PreInit),
            stk_ptr: Cell::new(STACK.as_ptr() as *mut u8),
            prd: PERIOD,
            leds: None,
            d: None,
        }
    }
}

impl Task for LedTask {
    fn run(&mut self) {
        let leds = self.leds.as_mut().unwrap();
        let mut d = self.d.as_mut().unwrap();
        for l in leds.iter_mut() {
            l.on();
            wait(&mut d, 10);
        }

        for l in leds.iter_mut().rev() {
            l.off();
            wait(&mut d, 10);
        }
    }

    fn get_stk_ptr(&self) -> *mut u8 {
        self.stk_ptr.get()
    }

    fn update_stk_ptr(&self, p: *mut u8) {
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
}

impl Led {
    pub fn off(&mut self) {
        self.pex.set_low().unwrap();
    }

    pub fn on(&mut self) {
        self.pex.set_high().unwrap();
    }
}

impl From<LedPin> for Led {
    fn from(pex: LedPin) -> Self {
        Led { pex }
    }
}

fn wait(d: &mut Timer<TIM2>, timeout: u32) {
    d.start(Hertz(timeout));
    while d.wait().is_err() {}
}

unsafe impl Sync for LedTask {}
