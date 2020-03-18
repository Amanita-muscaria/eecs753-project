#![no_std]
#![no_main]

extern crate rtos;
use f3::l3gd20::MODE as g_spi_mode;
use rtos::core::*;
use rtos::tasks::*;
use rtos::tasks::{accel::AccelTask, gyro::GyroTask, led::LedTask};

static mut GYRO: GyroTask = GyroTask::default();
static mut ACCEL: AccelTask = AccelTask::default();
static mut LEDS: LedTask = LedTask::default();

static mut TASKS: [&'static dyn Task; 3] = [unsafe { &GYRO }, unsafe { &ACCEL }, unsafe { &LEDS }];

static mut INT: Option<NVIC> = None;

#[entry]
unsafe fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let delay = Delay::new(cp.SYST, clocks);

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
    LEDS.init(p0, p1, p2, p3, p4, p5, p6, p7, delay);
    GYRO.init(spi, nss);

    INT = Some(cp.NVIC);

    loop {}
}

#[allow(non_snake_case)]
#[exception]
fn SysTick() {}

#[allow(non_snake_case)]
#[exception]
fn PendSV() {}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
