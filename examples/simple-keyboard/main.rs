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

pub mod hid;
pub mod keyboard;
pub mod matrix;

use crate::{matrix::Matrix, keyboard::Keyboard};
use cortex_m::asm::delay;
use embedded_hal::digital::v2::OutputPin;
use proton_c::led::Led;
use rtfm::{app, Instant};
use stm32f3xx_hal::{stm32, prelude::*, gpio::{AF14, gpioa::{PA11, PA12}}};
use stm32_usbd::UsbBus;
use usb_device::{class::UsbClass, bus, prelude::*};

type KeyboardHidClass = hid::HidClass<'static, UsbBus<(PA11<AF14>, PA12<AF14>)>, Keyboard>;
type Stm32F303UsbBus = UsbBus<(PA11<AF14>, PA12<AF14>)>;

// Generic keyboard from
// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
const VID: u16 = 0x27db;
const PID: u16 = 0x16c0;

#[app(device = stm32f3xx_hal::stm32)]
const APP: () = {
    static mut USB_DEV: UsbDevice<'static, Stm32F303UsbBus> = ();
    static mut USB_CLASS: KeyboardHidClass = ();
    static mut MATRIX: Matrix = ();

    #[init(schedule = [button_check, usb_poll])]
    fn init() -> init::LateResources {
        static mut USB_BUS: Option<bus::UsbBusAllocator<Stm32F303UsbBus>> = None;

        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();

        let clocks = rcc
            .cfgr
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .pclk2(24.mhz())
            .freeze(&mut flash.acr);

        let mut gpioa = device.GPIOA.split(&mut rcc.ahb);
        let mut gpiob = device.GPIOB.split(&mut rcc.ahb);
        let mut gpioc = device.GPIOC.split(&mut rcc.ahb);

        let mut led = Led::new(gpioc);
        led.on().unwrap();

        // let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        // usb_dp.set_low().unwrap();
        // delay(5 * 48);
        // usb_dp.set_high().unwrap();

        let usb_dm = gpioa.pa11.into_af14(&mut gpioa.moder, &mut gpioa.afrh);
        let usb_dp = gpioa.pa12.into_af14(&mut gpioa.moder, &mut gpioa.afrh);

        configure_usb_clock();

        *USB_BUS = Some(UsbBus::new(
            device.USB,
            (usb_dm, usb_dp)
        ));
        let usb_bus = USB_BUS.as_ref().unwrap();

        let usb_class = hid::HidClass::new(Keyboard::new(led), &usb_bus);
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(VID, PID))
            .manufacturer("dfrankland")
            .product("Proton-C")
            .serial_number(env!("CARGO_PKG_VERSION"))
            .build();

        schedule.button_check(Instant::now()).unwrap();
        schedule.usb_poll(Instant::now()).unwrap();

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
            USB_DEV: usb_dev,
            USB_CLASS: usb_class,
            MATRIX: matrix,
        }
    }

    #[task(schedule = [usb_poll], resources = [USB_DEV, USB_CLASS])]
    fn usb_poll() {
        dbg!("usb_poll");
        let usb_dev = resources.USB_DEV;
        resources.USB_CLASS.lock(|keyboard| {
            if usb_dev.poll(&mut [keyboard]) {
                keyboard.poll();
            }
        });
    }

    #[task(schedule = [button_check, usb_poll], resources = [MATRIX, USB_CLASS])]
    fn button_check() {
        dbg!("check pressed");
        if resources.MATRIX.pressed_keys()[0] {
            while let Ok(0) = resources.USB_CLASS.lock(|k| k.write(&[0, 0, 4_u8, 0, 0, 0, 0, 0])) {
                dbg!("pressed");
            }
        }

        schedule.button_check(Instant::now()).unwrap();
    }

    extern "C" {
        fn USART1_EXTI25();
    }
};

fn configure_usb_clock() {
    let rcc = unsafe { &*stm32::RCC::ptr() };
    rcc.cfgr.modify(|_, w| w.usbpre().set_bit());
}
