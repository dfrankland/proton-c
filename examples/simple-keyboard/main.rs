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

pub mod matrix;

use crate::matrix::Matrix;
use proton_c::led::Led;
use rtfm::{app, Instant};
use stm32f3xx_hal::prelude::*;

#[app(device = stm32f3xx_hal::stm32)]
const APP: () = {
    static mut MATRIX: Matrix = ();

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

        let matrix = Matrix::new(
            [
                Some(gpiob
                    .pb11
                    .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper)
                    .downgrade()
                    .downgrade())
            ],
            [
                Some(gpiob
                    .pb12
                    .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr)
                    .downgrade()
                    .downgrade())
            ],
        );

        init::LateResources {
            MATRIX: matrix,
        }
    }

    #[task(schedule = [button_check], resources = [MATRIX])]
    fn button_check() {
        dbg!(resources.MATRIX.pressed_keys());

        schedule.button_check(Instant::now()).unwrap();
    }

    extern "C" {
        fn USART1_EXTI25();
    }
};
