#![no_std]
#![no_main]

extern crate panic_semihosting;

pub mod hid;
pub mod keyboard;
pub mod matrix;

use crate::{keyboard::Keyboard, matrix::Matrix};
use cortex_m::asm::delay;
use embedded_hal::digital::v2::OutputPin;
use proton_c::led::Led;
use rtfm::app;
use stm32_usbd::{UsbBus, UsbPinsType};
use stm32f3xx_hal::{
    gpio::{
        gpioa::{PA11, PA12},
        AF14,
    },
    prelude::*,
    stm32, timer,
};
use usb_device::{bus, class::UsbClass, prelude::*};

type KeyboardHidClass = hid::HidClass<'static, UsbBus<UsbPinsType>, Keyboard>;
type Stm32F303UsbBus = UsbBus<UsbPinsType>;

// Generic keyboard from
// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
const VID: u16 = 0x27db;
const PID: u16 = 0x16c0;

#[app(device = stm32f3xx_hal::stm32)]
const APP: () = {
    static mut USB_DEV: UsbDevice<'static, Stm32F303UsbBus> = ();
    static mut USB_CLASS: KeyboardHidClass = ();
    static mut TIMER: timer::Timer<stm32::TIM3> = ();
    static mut MATRIX: Matrix = ();

    #[init]
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
        let gpioc = device.GPIOC.split(&mut rcc.ahb);

        let mut led = Led::new(gpioc);
        led.on().expect("Couldn't turn the LED on!");

        // Pull the D+ pin down to send a RESET condition to the USB bus.
        let mut usb_dp = gpioa
            .pa12
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        usb_dp.set_low().expect("Couldn't reset the USB bus!");
        delay(clocks.sysclk().0 / 100);

        let usb_dm = gpioa.pa11.into_af14(&mut gpioa.moder, &mut gpioa.afrh);
        let usb_dp = usb_dp.into_af14(&mut gpioa.moder, &mut gpioa.afrh);

        configure_usb_clock();

        *USB_BUS = Some(UsbBus::new(device.USB, (usb_dm, usb_dp)));
        let usb_bus = USB_BUS
            .as_ref()
            .expect("Couldn't make the USB_BUS a static reference!");

        let usb_class = hid::HidClass::new(Keyboard::new(led), &usb_bus);
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(VID, PID))
            .manufacturer("dfrankland")
            .product("Proton-C")
            .serial_number(env!("CARGO_PKG_VERSION"))
            .device_class(3)
            .build();

        let mut timer = timer::Timer::tim3(device.TIM3, 1.khz(), clocks, &mut rcc.apb1);
        timer.listen(timer::Event::Update);

        init::LateResources {
            USB_DEV: usb_dev,
            USB_CLASS: usb_class,
            TIMER: timer,
            MATRIX: Matrix::new(
                [Some(
                    gpiob
                        .pb11
                        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper)
                        .downgrade()
                        .downgrade(),
                )],
                [Some(
                    gpiob
                        .pb12
                        .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr)
                        .downgrade()
                        .downgrade(),
                )],
            ),
        }
    }

    #[interrupt(priority = 2, resources = [USB_DEV, USB_CLASS])]
    fn USB_HP_CAN_TX() {
        usb_poll(&mut resources.USB_DEV, &mut resources.USB_CLASS);
    }

    #[interrupt(priority = 2, resources = [USB_DEV, USB_CLASS])]
    fn USB_LP_CAN_RX0() {
        usb_poll(&mut resources.USB_DEV, &mut resources.USB_CLASS);
    }

    #[interrupt(priority = 1, resources = [USB_DEV, USB_CLASS, MATRIX, TIMER])]
    fn TIM3() {
        resources.TIMER.clear_update_interrupt_flag();

        let key_pressed = resources
            .MATRIX
            .pressed_keys()
            .expect("Couldn't poll pressed keys!")[0][0];
        resources
            .USB_CLASS
            .lock(|k| {
                // Type the character `a`
                if key_pressed {
                    k.write(&[0, 0, 4, 0, 0, 0, 0, 0])
                } else {
                    k.write(&[0, 0, 0, 0, 0, 0, 0, 0])
                }
            })
            .expect("Couldn't get access to USB_CLASS!");
    }
};

fn configure_usb_clock() {
    let rcc = unsafe { &*stm32::RCC::ptr() };
    rcc.cfgr.modify(|_, w| w.usbpre().set_bit());
}

fn usb_poll(
    usb_dev: &mut UsbDevice<'static, UsbBus<(PA11<AF14>, PA12<AF14>)>>,
    keyboard: &mut KeyboardHidClass,
) {
    if usb_dev.poll(&mut [keyboard]) {
        keyboard.poll();
    }
}
