use crate::hal::gpio::{gpioc, Output, PushPull};
use core::ops::{Deref, DerefMut};

/// Abstraction for the only LED on the board
pub struct Led {
    pcx: gpioc::PCx<Output<PushPull>>,
}

impl Led {
    /// Initializes the LED
    pub fn new(mut gpioc: gpioc::Parts) -> Self {
        let led = gpioc
            .pc13
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

        led.into()
    }
}

impl Deref for Led {
    type Target = gpioc::PCx<Output<PushPull>>;

    fn deref(&self) -> &Self::Target {
        &self.pcx
    }
}

impl DerefMut for Led {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pcx
    }
}

/// The only LED on the board
pub type LED = gpioc::PC13<Output<PushPull>>;

impl Into<Led> for LED {
    fn into(self) -> Led {
        Led {
            pcx: self.downgrade(),
        }
    }
}
