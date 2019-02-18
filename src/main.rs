#![no_std]
#![no_main]

use cortex_m::asm;
use cortex_m_rt::entry;
use panic_halt;
use stm32f3::stm32f303;

#[entry]
fn main() -> ! {
    // get handles to the hardware
    let peripherals = stm32f303::Peripherals::take().unwrap();
    let gpioc = &peripherals.GPIOC;
    let rcc = &peripherals.RCC;

    // enable the GPIO clock for IO port C
    rcc.ahbenr.write(|w| w.iopcen().set_bit());
    gpioc.moder.write(|w| w.moder13().output());
    gpioc.ospeedr.write(|w| w.ospeedr13().very_high_speed());

    loop {
        gpioc.bsrr.write(|w| w.bs13().set_bit());
        asm::delay(2_000_000);
        gpioc.brr.write(|w| w.br13().set_bit());
        asm::delay(2_000_000);
    }
}
