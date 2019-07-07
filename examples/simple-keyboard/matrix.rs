use cortex_m::asm::delay;
use embedded_hal::digital::v2::{OutputPin, InputPin};
use stm32f3xx_hal::gpio::{PXx, Output, Input, PushPull, PullUp};

macro_rules! matrix {
    ($rows:tt, $cols:tt) => {
        pub struct Matrix {
            rows: [Option<PXx<Output<PushPull>>>; $rows],
            cols: [Option<PXx<Input<PullUp>>>; $cols],
        }

        impl Matrix {
            pub fn new(
                rows: [Option<PXx<Output<PushPull>>>; $rows],
                cols: [Option<PXx<Input<PullUp>>>; $cols],
            ) -> Self {
                Self { rows, cols }
            }

            pub fn pressed_keys(&mut self) -> Result<[[bool; $cols]; $rows], ()> {
                let mut pressed_keys = [[false; $cols]; $rows];

                for (row_index, col_option) in self.rows.iter_mut().enumerate() {
                    if let Some(col) = col_option {
                        col.set_low()?;
                        delay(5 * 48); // 5µs

                        for (col_index, row_option) in self.cols.iter().enumerate() {
                            let mut pressed = false;
                            if let Some(row) = row_option {
                                pressed = row.is_low()?;
                            }
                            pressed_keys[row_index][col_index] = pressed;
                        }

                        col.set_high()?;
                    }
                }

                Ok(pressed_keys)
            }
        }
    }
}

matrix!(1, 1);
