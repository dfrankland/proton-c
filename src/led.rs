use embedded_hal::digital::v2::OutputPin;
use crate::hal::gpio::{gpioc, Output, PushPull};

/// Abstraction for the only LED on the board
pub struct Led {
    pcx: gpioc::PCx<Output<PushPull>>,
    on: bool,
}

impl Led {
    /// Initializes the LED
    pub fn new(mut gpioc: gpioc::Parts) -> Self {
        let led = gpioc
            .pc13
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

        led.into()
    }

    /// Turns the LED off
    pub fn off(&mut self) -> Result<(), ()> {
        self.on = false;
        self.pcx.set_low()
    }

    /// Turns the LED on
    pub fn on(&mut self) -> Result<(), ()> {
        self.on = true;
        self.pcx.set_high()
    }

    /// Check the LED
    pub fn is_on(&self) -> bool {
        self.on
    }
}

/// The only LED on the board
pub type LED = gpioc::PC13<Output<PushPull>>;

impl Into<Led> for LED {
    fn into(self) -> Led {
        Led {
            pcx: self.downgrade(),
            on: false,
        }
    }
}
