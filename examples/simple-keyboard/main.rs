#![no_std]
#![no_main]

extern crate panic_semihosting;

#[allow(unused)]
macro_rules! dbg {
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                use core::fmt::Write;
                let mut out = cortex_m_semihosting::hio::hstdout().unwrap();
                writeln!(
                    out,
                    "[{}:{}] {} = {:#?}",
                    file!(),
                    line!(),
                    stringify!($val),
                    &tmp
                )
                .unwrap();
                tmp
            }
        }
    };
}

use cortex_m::asm::delay;
use embedded_hal::digital::v2::{OutputPin, InputPin};
use proton_c::led::Led;
use rtfm::{app, Instant};
use stm32f3xx_hal::{
    prelude::*,
    gpio::{gpiob, Output, Input, PushPull, PullUp},
};

#[app(device = stm32f3xx_hal::stm32)]
const APP: () = {
    static mut MATRIX: Matrix<gpiob::PBx<Output<PushPull>>, gpiob::PBx<Input<PullUp>>> = ();

    #[init(schedule = [button_check])]
    fn init() -> init::LateResources {
        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();

        rcc
            .cfgr
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .pclk2(24.mhz())
            .freeze(&mut flash.acr);

        let mut gpiob = device.GPIOB.split(&mut rcc.ahb);
        let gpioc = device.GPIOC.split(&mut rcc.ahb);

        let mut led = Led::new(gpioc);
        led.on().unwrap();

        schedule.button_check(Instant::now()).unwrap();

        init::LateResources {
            MATRIX: Matrix {
                rows: [
                    gpiob
                        .pb11
                        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper)
                        .downgrade()
                ],
                cols: [
                    gpiob
                        .pb12
                        .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr)
                        .downgrade()
                ],
            },
        }
    }

    #[task(schedule = [button_check], resources = [MATRIX])]
    fn button_check() {
        let mut cols = [false; 1];

        for c in resources.MATRIX.rows.iter_mut() {
            c.set_low().unwrap();
            delay(5 * 48); // 5µs
            for (index, r) in resources.MATRIX.cols.iter().enumerate() {
                cols[index] = r.is_low().unwrap();
            }
            c.set_high().unwrap();
        }

        dbg!(cols);

        schedule.button_check(Instant::now()).unwrap();
    }

    extern "C" {
        fn USART1_EXTI25();
    }
};

pub struct Matrix<T: OutputPin, U: InputPin> {
    pub rows: [T; 1],
    pub cols: [U; 1],
}
