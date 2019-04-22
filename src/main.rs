#![no_std]
#![no_main]

extern crate panic_semihosting;

use embedded_hal::digital::{InputPin, OutputPin};
use rtfm::{app, Instant};
use stm32f3xx_hal::{
    gpio::{gpiob, gpioc, GpioExt, Input, Output, PullUp, PushPull},
    rcc::RccExt,
};

#[app(device = stm32f3xx_hal::stm32)]
const APP: () = {
    static mut BUTTON: gpiob::PB11<Input<PullUp>> = ();
    static mut LED: gpioc::PC13<Output<PushPull>> = ();

    #[init(schedule = [button_check])]
    fn init() -> init::LateResources {
        let mut rcc = device.RCC.constrain();

        let mut gpiob = device.GPIOB.split(&mut rcc.ahb);
        let mut gpioc = device.GPIOC.split(&mut rcc.ahb);

        let button = gpiob
            .pb11
            .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr);
        let led = gpioc
            .pc13
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

        schedule.button_check(Instant::now()).unwrap();

        init::LateResources {
            LED: led,
            BUTTON: button,
        }
    }

    #[task(schedule = [button_check], resources = [BUTTON, LED])]
    fn button_check() {
        if resources.BUTTON.is_low() {
            resources.LED.set_high();
        } else {
            resources.LED.set_low();
        }

        schedule.button_check(Instant::now()).unwrap();
    }

    extern "C" {
        fn USART1_EXTI25();
    }
};
