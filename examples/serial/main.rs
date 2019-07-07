//! CDC-ACM serial port example using cortex-m-rtfm.
#![no_main]
#![no_std]

extern crate panic_semihosting;

mod cdc_acm;

use cortex_m::asm::delay;
use rtfm::app;
use stm32f3xx_hal::{stm32, prelude::*};
use embedded_hal::digital::v2::OutputPin;
use stm32_usbd::{UsbBus, UsbBusType};
use usb_device::{bus, prelude::*};

#[app(device = stm32f3xx_hal::stm32)]
const APP: () = {
    static mut USB_DEV: UsbDevice<'static, UsbBusType> = ();
    static mut SERIAL: cdc_acm::SerialPort<'static, UsbBusType> = ();

    #[init]
    fn init() {
        static mut USB_BUS: Option<bus::UsbBusAllocator<UsbBusType>> = None;

        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();

        let clocks = rcc
            .cfgr
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .pclk2(24.mhz())
            .freeze(&mut flash.acr);

        let mut gpioa = device.GPIOA.split(&mut rcc.ahb);
        let mut gpioc = device.GPIOC.split(&mut rcc.ahb);

        let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);
        led.set_high().unwrap();

        // Proton C board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        usb_dp.set_low().unwrap();
        delay(clocks.sysclk().0 / 100);

        let usb_dm = gpioa.pa11.into_af14(&mut gpioa.moder, &mut gpioa.afrh);
        let usb_dp = usb_dp.into_af14(&mut gpioa.moder, &mut gpioa.afrh);

        configure_usb_clock();

        *USB_BUS = Some(UsbBus::new(device.USB, (usb_dm, usb_dp)));

        let serial = cdc_acm::SerialPort::new(USB_BUS.as_ref().unwrap());

        let usb_dev =
            UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0x5824, 0x27dd))
                .manufacturer("Fake company")
                .product("Serial port")
                .serial_number("TEST")
                .device_class(cdc_acm::USB_CLASS_CDC)
                .build();

        USB_DEV = usb_dev;
        SERIAL = serial;
    }

    #[interrupt(resources = [USB_DEV, SERIAL])]
    fn USB_HP_CAN_TX() {
        usb_poll(&mut resources.USB_DEV, &mut resources.SERIAL);
    }

    #[interrupt(resources = [USB_DEV, SERIAL])]
    fn USB_LP_CAN_RX0() {
        usb_poll(&mut resources.USB_DEV, &mut resources.SERIAL);
    }
};

fn usb_poll<B: bus::UsbBus>(
    usb_dev: &mut UsbDevice<'static, B>,
    serial: &mut cdc_acm::SerialPort<'static, B>,
) {
    if !usb_dev.poll(&mut [serial]) {
        return;
    }

    let mut buf = [0u8; 8];

    match serial.read(&mut buf) {
        Ok(count) if count > 0 => {
            // Echo back in upper case
            for c in buf[0..count].iter_mut() {
                if 0x61 <= *c && *c <= 0x7a {
                    *c &= !0x20;
                }
            }

            serial.write(&buf[0..count]).ok();
        }
        _ => {}
    }
}

fn configure_usb_clock() {
    let rcc = unsafe { &*stm32::RCC::ptr() };
    rcc.cfgr.modify(|_, w| w.usbpre().set_bit());
}
