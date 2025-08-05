#![no_main]
#![no_std]

use rmk::macros::rmk_peripheral;
use panic_probe as _;

#[rmk_peripheral(id = 0)]
mod keyboard_peripheral {}
