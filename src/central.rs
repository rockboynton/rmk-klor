#![no_main]
#![no_std]

use panic_probe as _;
use rmk::macros::rmk_central;

mod display;

#[rmk_central]
mod keyboard_central {}
