#![no_std]
#![no_main]

extern crate panic_semihosting;

use proton_c::led::Led;
use rtfm::{app, Instant};
use stm32f3xx_hal::{prelude::*, gpio::GpioExt, rcc::RccExt};

const PERIOD: u32 = 2_000_000;

#[app(device = stm32f3xx_hal::stm32)]
const APP: () = {
    static mut LED: Led = ();

    #[init(schedule = [led_toggle])]
    fn init() -> init::LateResources {
        let mut rcc = device.RCC.constrain();

        let gpioc = device.GPIOC.split(&mut rcc.ahb);
        let led = Led::new(gpioc);

        schedule.led_toggle(Instant::now()).unwrap();

        init::LateResources { LED: led }
    }

    #[task(schedule = [led_toggle], resources = [LED])]
    fn led_toggle() {
        if resources.LED.is_set_low().unwrap() {
            resources.LED.set_high().unwrap();
        } else {
            resources.LED.set_low().unwrap();
        }
        schedule.led_toggle(scheduled + PERIOD.cycles()).unwrap();
    }

    extern "C" {
        fn USART1_EXTI25();
    }
};
