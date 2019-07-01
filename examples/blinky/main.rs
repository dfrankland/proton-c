#![no_std]
#![no_main]

extern crate panic_semihosting;

use proton_c::led::Led;
use rtfm::{app, Instant};
use stm32f3xx_hal::{
    gpio::GpioExt,
    rcc::RccExt,
};

const PERIOD: u32 = 2_000_000;

#[app(device = stm32f3xx_hal::stm32)]
const APP: () = {
    static mut LED: Led = ();

    #[init(schedule = [led_on])]
    fn init() -> init::LateResources {
        let mut rcc = device.RCC.constrain();

        let gpioc = device.GPIOC.split(&mut rcc.ahb);
        let led = Led::new(gpioc);

        schedule.led_on(Instant::now()).unwrap();

        init::LateResources { LED: led }
    }

    #[task(schedule = [led_off], resources = [LED])]
    fn led_on() {
        resources.LED.on().unwrap();
        schedule.led_off(scheduled + PERIOD.cycles()).unwrap();
    }

    #[task(schedule = [led_on], resources = [LED])]
    fn led_off() {
        resources.LED.off().unwrap();
        schedule.led_on(scheduled + PERIOD.cycles()).unwrap();
    }

    extern "C" {
        fn USART1_EXTI25();
    }
};
