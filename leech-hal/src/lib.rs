#![no_std]

pub use litex_pac as pac;

pub mod interface;

#[cfg(with_ps2)]
pub mod ps2;
