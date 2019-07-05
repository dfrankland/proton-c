use cortex_m::asm::delay;
use embedded_hal::digital::v2::{OutputPin, InputPin};
use stm32f3xx_hal::gpio::{PXx, Output, Input, PushPull, PullUp};

pub struct Matrix {
    rows: [Option<PXx<Output<PushPull>>>; 1],
    cols: [Option<PXx<Input<PullUp>>>; 1],
}

impl Matrix {
    pub fn new(rows: [Option<PXx<Output<PushPull>>>; 1], cols: [Option<PXx<Input<PullUp>>>; 1]) -> Self {
        Self { rows, cols }
    }

    pub fn pressed_keys(&mut self) -> [bool; 1] {
        let mut cols = [false; 1];

        for col_option in self.rows.iter_mut() {
            if let Some(col) = col_option {
                col.set_low().unwrap();
                delay(5 * 48); // 5µs
                for (index, row_option) in self.cols.iter().enumerate() {
                    let mut pressed = false;
                    if let Some(row) = row_option {
                        pressed = row.is_low().unwrap();
                    }
                    cols[index] = pressed;
                }
                col.set_high().unwrap();
            }
        }

        cols
    }
}
