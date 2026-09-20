#![no_main]
#![no_std]

use panic_probe as _;
use rmk::macros::rmk_peripheral;

mod display;

// RMK 0.9's split-peripheral macro imports this name even when
// `dependency.defmt_log = false`; this empty module keeps defmt absent.
mod defmt_rtt {}

#[rmk_peripheral(id = 0)]
mod keyboard_peripheral {}
