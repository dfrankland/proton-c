use cortex_m::asm::delay;
use embedded_hal::digital::v2::{OutputPin, InputPin};
use stm32f3xx_hal::gpio::{gpiob, Output, Input, PushPull, PullUp};

pub struct Matrix<T: OutputPin, U: InputPin> {
    pub rows: [T; 1],
    pub cols: [U; 1],
}

impl<T: OutputPin<Error=()>, U: InputPin<Error=()>> Matrix<T, U> {
    pub fn pressed_keys(&mut self) -> [bool; 1] {
        let mut cols = [false; 1];

        for c in self.rows.iter_mut() {
            c.set_low().unwrap();
            delay(5 * 48); // 5µs
            for (index, r) in self.cols.iter().enumerate() {
                cols[index] = r.is_low().unwrap();
            }
            c.set_high().unwrap();
        }

        cols
    }
}

pub type MatrixGpioB = Matrix<gpiob::PBx<Output<PushPull>>, gpiob::PBx<Input<PullUp>>>;
