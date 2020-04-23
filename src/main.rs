#![no_std]
#![no_main]
#![feature(let_chains)]

extern crate rtos;
use f3::l3gd20::MODE as g_spi_mode;
use rtos::core::*;
use rtos::core::{CorePeripherals, DevPeripherals, Timer, SCB, SYST, TIM2};
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
        Timer::tim2(dp.TIM2, Hertz(0), clocks, &mut rcc.apb1),
    );
    GYRO.init(spi, nss);

    TASK_MAN = Some(TaskMan::new(TASKS));

    cp.SYST.set_reload(TASKS[0].get_prd());
    cp.SYST.clear_current();
    cp.SYST.enable_counter();
    SYSTICK = Some(cp.SYST);

    SCB::set_pendsv();
    loop {
        wfi();
    }
}

#[allow(non_snake_case)]
#[exception]
unsafe fn SysTick() {
    let mut tm = TASK_MAN.take().unwrap();
    let current = tm.get_pri(TaskID::Current);
    let next = tm.get_pri(TaskID::NextRelease).unwrap();
    tm.release_next();

    TASK_MAN.replace(tm);

    if let Some(c) = current {
        if c > next {
            SCB::set_pendsv();
        }
    }
}

#[allow(non_snake_case)]
#[exception]
unsafe fn PendSV() {
    let tm = TASK_MAN.take().unwrap();
    let current = tm.get(TaskID::Current);
    let next = if let Some(i) = tm.next_to_run() {
        tm.get(TaskID::ID(i))
    } else {
        None
    };

    if let Some(c) = current {
        //save into current task
    }

    TASK_MAN.replace(tm);
}

unsafe fn task_done() {
    let mut tm = TASK_MAN.take().unwrap();
    tm.set_state(TaskID::Current, TaskState::Done);
    tm.set_next_release(TaskID::Current, SYST::get_current());

    TASK_MAN.replace(tm);

    SCB::set_pendsv();
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
