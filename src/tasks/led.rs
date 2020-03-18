use crate::core::{Delay, DelayUs, Output, OutputPin, PEx, PushPull};
use crate::tasks::{Task, TaskState};

const STK_SIZE: usize = 512;
const PERIOD: usize = 0;
const STACK: [u8; STK_SIZE] = [0; STK_SIZE];

type LedPin = PEx<Output<PushPull>>;

pub struct LedTask {
    state: TaskState,
    stk_ptr: *mut u8,
    prd: usize,
    leds: Option<[Led; 8]>,
    d: Option<Delay>,
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
        d: Delay,
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
            state: TaskState::Ready,
            stk_ptr: STACK.as_ptr() as *mut u8,
            prd: PERIOD,
            leds: None,
            d: None,
        }
    }
}

impl Task for LedTask {
    fn run(&mut self) {
        let leds = self.leds.as_mut().unwrap();
        let d = self.d.as_mut().unwrap();
        for l in leds.iter_mut() {
            l.on();
            d.delay_us(100_u8);
        }

        for l in leds.iter_mut().rev() {
            l.off();
            d.delay_us(100_u8);
        }
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
