#![no_std]
#![no_main]
#![feature(llvm_asm)]

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
        Timer::tim2(dp.TIM2, Hertz(1), clocks, &mut rcc.apb1),
    );
    GYRO.init(spi, nss);

    TASK_MAN = Some(TaskMan::new(TASKS));
    // let t = SYST::get_ticks_per_10ms() / 100 * TASKS[0].get_prd();
    cp.SYST.set_reload(25);
    cp.SYST.clear_current();
    cp.SYST.enable_counter();
    cp.SYST.enable_interrupt();
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
    let running = tm.get_pri(TaskID::Current);
    let released = tm.get_pri(TaskID::NextRelease).unwrap();
    tm.release_next();

    // let t = SYST::get_ticks_per_10ms();
    // let next_tick = tm.get_pri(TaskID::NextRelease).unwrap();
    // st.set_reload(next_tick * t);
    // st.clear_current();
    // st.enable_counter();
    SYSTICK.replace(st);
    TASK_MAN.replace(tm);

    if let Some(r) = running {
        if r > released {
            SCB::set_pendsv();
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
#[exception]
unsafe fn PendSV() {
    let tm = TASK_MAN.take().unwrap();
    let current = if let Some(c) = tm.get(TaskID::Current) {
        Some(c.get_stk_ptr())
    } else {
        None
    };
    let next = if let Some(i) = tm.next_to_run() {
        let n = tm.get(TaskID::ID(i)).unwrap();
        Some(n.get_stk_ptr())
    } else {
        None
    };
    TASK_MAN.replace(tm);

    if let Some(mut c) = current {
        llvm_asm! {"
        push {$0}
        mrs r0, psp
        isb
        stmdb r0!, {r1-r12}
        pop {r1}
        str r0, [r1]
        "
        :
        : "r"(c)
        }
    }

    if let Some(n) = next {
        llvm_asm! {"
        add r1, $0, #56
        msr psp, r1
        mov sp, $0
        ldmia sp!, {r0-r12} 
        ldr lr, [sp, #4]
        add sp, sp, #4
        bx lr
        "
        :
        : "r"(n)
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
#[exception]
unsafe fn SVCall() {
    SCB::set_pendsv();
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
