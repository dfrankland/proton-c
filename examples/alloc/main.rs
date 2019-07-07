#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;
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

use proton_c::led::Led;
use rtfm::{app, Instant};
use stm32f3xx_hal::prelude::*;
use alloc::{vec::Vec, string::String};
use alloc_cortex_m::CortexMHeap;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[app(device = stm32f3xx_hal::stm32)]
const APP: () = {
    static mut MESSAGES: Vec<String> = ();

    #[init(schedule = [ping])]
    fn init() -> init::LateResources {
        let start = cortex_m_rt::heap_start() as usize;
        let size = 1024 * 30; // 30 KB
        unsafe { ALLOCATOR.init(start, size) }

        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();

        rcc
            .cfgr
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .pclk2(24.mhz())
            .freeze(&mut flash.acr);

        let gpioc = device.GPIOC.split(&mut rcc.ahb);

        let mut led = Led::new(gpioc);
        led.on().unwrap();

        schedule.ping(Instant::now()).unwrap();

        init::LateResources {
            MESSAGES: Vec::new(),
        }
    }

    #[task(schedule = [pong], resources = [MESSAGES])]
    fn ping() {
        resources.MESSAGES.push(String::from("ping"));

        dbg!(resources.MESSAGES);

        schedule.pong(Instant::now()).unwrap();
    }

    #[task(schedule = [ping], resources = [MESSAGES])]
    fn pong() {
        resources.MESSAGES.push(String::from("pong"));

        dbg!(resources.MESSAGES);

        schedule.ping(Instant::now()).unwrap();
    }

    extern "C" {
        fn USART1_EXTI25();
    }
};

// Define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();

    #[allow(clippy::empty_loop)]
    loop {}
}
