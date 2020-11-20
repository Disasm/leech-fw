#![no_main]
#![no_std]

use panic_rtt_target as _;

use cortex_m_rt::entry;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::stm32;
use rtt_target::{rtt_init_print, rprintln};
use leech_hal::pac::read_reg;
use leech_hal::pac::{ctrl, ps2};
use leech_hal::interface::SpiMemoryInterface;
use leech_hal::ps2::PS2;
use core::slice;

const BITSTREAM: &[u8] = include_bytes!("../../soc/build/leech/gateware/leech.bin");

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let dp = stm32::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(25.mhz()).sysclk(96.mhz()).freeze();

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();

    let mut led = gpioc.pc13.into_open_drain_output();

    let mut mem_interface = SpiMemoryInterface::init(
        dp.SPI2,
        gpiob.pb12, // CS
        gpiob.pb13, // SCK
        gpiob.pb14, // MISO
        gpiob.pb15, // MOSI
        gpiob.pb2, // CRESET
        gpioa.pa8, // MCO
        gpiob.pb10, // IRQ
        clocks
    );

    led.set_low().ok();
    rprintln!("Starting bitstream upload...");
    mem_interface.upload_bitstream(BITSTREAM);
    rprintln!("Bitstream upload finished");
    led.set_high().ok();

    mem_interface.install();

    let ctrl = ctrl::CTRL::take().unwrap();
    assert_eq!(read_reg!(ctrl, ctrl, SCRATCH), 0x12345678);

    let ps2 = ps2::PS2::take().unwrap();
    let mut ps2 = PS2::new(ps2);

    loop {
        let mut byte = 0;
        if ps2.read(slice::from_mut(&mut byte)) != 0 {
            rprintln!("key {:02x}", byte);
        }
    }
}
