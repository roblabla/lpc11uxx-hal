//! [Hardware Abstraction Layer](https://crates.io/crates/embedded-hal) (HAL)
//! for NXP LPC11UXX family of µ-controllers.
#![no_std]
#![feature(const_fn)]

pub use lpc11uxx;

#[macro_use]
mod const_shenanigans;

pub mod clock;
pub mod delay;
pub mod gpio;
pub mod serial;
//pub mod spi;