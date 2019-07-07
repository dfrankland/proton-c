#![no_std]
#![no_main]

extern crate panic_semihosting;

use embedded_hal::digital::v2::InputPin;
use proton_c::led::Led;
use rtfm::{app, Instant};
use stm32f3xx_hal::{
    gpio::{gpiob, GpioExt, Input, PullUp},
    rcc::RccExt,
};

#[app(device = stm32f3xx_hal::stm32)]
const APP: () = {
    static mut BUTTON: gpiob::PB11<Input<PullUp>> = ();
    static mut LED: Led = ();

    #[init(schedule = [button_check])]
    fn init() -> init::LateResources {
        let mut rcc = device.RCC.constrain();

        let mut gpiob = device.GPIOB.split(&mut rcc.ahb);
        let gpioc = device.GPIOC.split(&mut rcc.ahb);

        let button = gpiob
            .pb11
            .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr);
        let led = Led::new(gpioc);

        schedule.button_check(Instant::now()).unwrap();

        init::LateResources {
            LED: led,
            BUTTON: button,
        }
    }

    #[task(schedule = [button_check], resources = [BUTTON, LED])]
    fn button_check() {
        if resources.BUTTON.is_low().unwrap() {
            resources.LED.on().unwrap();
        } else {
            resources.LED.off().unwrap();
        }

        schedule.button_check(Instant::now()).unwrap();
    }

    extern "C" {
        fn USART1_EXTI25();
    }
};
