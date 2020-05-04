#![no_std]
#![no_main]
#![feature(llvm_asm)]

extern crate rtos;
use cortex_m::asm::bkpt;
use f3::l3gd20::MODE as g_spi_mode;
use rtos::core::*;
use rtos::core::{CorePeripherals, DevPeripherals, Timer, SCB, SYST};
use rtos::task_man::{TaskID, TaskMan};
use rtos::tasks::*;
use rtos::tasks::{accel::AccelTask, gyro::GyroTask, led::LedTask};

static mut GYRO: GyroTask = GyroTask::default();
static mut ACCEL: AccelTask = AccelTask::default();
static mut LEDS: LedTask = LedTask::default();

static TASKS: [&dyn Task; 3] = [unsafe { &GYRO }, unsafe { &ACCEL }, unsafe { &LEDS }];

static mut SYSTICK: Option<SYST> = None;

static mut TASK_MAN: Option<TaskMan> = None;

#[entry]
unsafe fn main() -> ! {
    let mut cp = CorePeripherals::take().unwrap();
    let dp = DevPeripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);

    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 400.khz(), clocks, &mut rcc.apb1);

    let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        g_spi_mode,
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );
    let nss = gpioe
        .pe3
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    let p0 = gpioe
        .pe8
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
        .downgrade();
    let p1 = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
        .downgrade();
    let p2 = gpioe
        .pe10
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
        .downgrade();
    let p3 = gpioe
        .pe11
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
        .downgrade();
    let p4 = gpioe
        .pe12
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
        .downgrade();
    let p5 = gpioe
        .pe13
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
        .downgrade();
    let p6 = gpioe
        .pe14
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
        .downgrade();
    let p7 = gpioe
        .pe15
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
        .downgrade();
    ACCEL.init(i2c);
    LEDS.init(
        p0,
        p1,
        p2,
        p3,
        p4,
        p5,
        p6,
        p7,
        Timer::tim2(dp.TIM2, Hertz(1), clocks, &mut rcc.apb1),
    );
    GYRO.init(spi, nss);

    let ticks = SYST::get_ticks_per_10ms() / 10;
    let tm = TaskMan::new(TASKS, ticks);
    let t = tm.next_release_time();
    cp.SYST.set_reload(t);
    cp.SYST.clear_current();
    cp.SYST.enable_interrupt();
    cp.SYST.enable_counter();
    TASK_MAN = Some(tm);
    SYSTICK = Some(cp.SYST);

    SCB::set_pendsv();

    loop {
        wfe();
    }
}

#[allow(non_snake_case)]
#[exception]
unsafe fn SysTick() {
    let mut st = SYSTICK.take().unwrap();
    st.disable_counter();

    let mut tm = TASK_MAN.take().unwrap();

    tm.update_release(SYST::get_reload());

    let pend = if let Some(c) = tm.get_pri(TaskID::Current) {
        if let Some(n) = tm.get_pri(TaskID::Next) {
            n < c
        } else {
            false
        }
    } else {
        true
    };

    let next = tm.next_release_time();

    TASK_MAN.replace(tm);

    st.clear_current();
    st.set_reload(next);
    st.enable_counter();

    SYSTICK.replace(st);

    if pend {
        SCB::set_pendsv();
    }
}

#[no_mangle]
#[allow(non_snake_case)]
#[exception]
unsafe fn SVCall() {
    let mut _r12 = 0;
    llvm_asm! {"
    mov $0, r12
    "
    : "=r"(_r12)
    :
    : "$0"
    : "volatile"
    }
    let tm = TASK_MAN.take().unwrap();
    let t = tm.get(TaskID::ID(_r12)).unwrap();
    t.update_stk_ptr(save());
    t.set_state(TaskState::Done);
    TASK_MAN.replace(tm);
    SCB::set_pendsv();
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    bkpt();
    loop {}
}

#[no_mangle]
#[allow(non_snake_case)]
#[exception]
#[inline(always)]
unsafe fn PendSV() {
    let tm = TASK_MAN.take().unwrap();
    let current = tm.get(TaskID::Current);

    let next = if let Some(i) = tm.next_to_run() {
        let n = tm.get(TaskID::ID(i)).unwrap();
        n.set_state(TaskState::Running);
        Some(n.get_stk_ptr())
    } else {
        None
    };

    if let Some(c) = current {
        c.update_stk_ptr(save());
        c.set_state(TaskState::Suspended);
    }

    TASK_MAN.replace(tm);

    if let Some(n) = next {
        llvm_asm! {"
        ldmia $0!, {r4-r11}
        msr psp, $0
        isb
        mov r6, 0xFFFFFFFD
        bx r6
        "
        :
        : "r"(n)
        : "$0", "r1", "r6"
        : "volatile"
        }
    }
}

#[inline(always)]
unsafe fn save() -> *mut u32 {
    let mut _sp: *mut u32 = 0 as *mut u32;
    llvm_asm! {"
        mrs r1, psp
        isb
        stmdb r1!, {r4-r11}
        mov $0, r1
        "
    : "=r"(_sp)
    :
    : "r1", "$0", "r6"
    : "volatile"
    }
    _sp
}
