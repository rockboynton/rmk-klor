#![no_main]
#![no_std]

use rmk::macros::rmk_central;
use panic_probe as _;

#[rmk_central]
mod keybaord_central {}
