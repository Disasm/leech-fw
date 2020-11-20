#![no_main]
#![no_std]

use panic_rtt_target as _;

use cortex_m_rt::entry;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::stm32;
use stm32f4xx_hal::interrupt;
use rtt_target::{rtt_init_print, rprintln};
use leech_hal::pac::read_reg;
use leech_hal::pac::{ctrl, ps2};
use leech_hal::interface::SpiMemoryInterface;
use leech_hal::ps2::PS2;

const BITSTREAM: &[u8] = include_bytes!("../../soc/build/leech/gateware/leech.bin");

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut dp = stm32::Peripherals::take().unwrap();
    dp.RCC.apb2enr.write(|w| w.syscfgen().enabled());

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
    mem_interface.irq_listen(&mut dp.EXTI, &mut dp.SYSCFG);

    let ctrl = ctrl::CTRL::take().unwrap();
    assert_eq!(read_reg!(ctrl, ctrl, SCRATCH), 0x12345678);

    let ps2 = ps2::PS2::take().unwrap();
    let mut ps2 = PS2::new(ps2);
    ps2.listen();

    loop { continue; }
}

#[interrupt]
fn EXTI15_10() {
    // We cheat and don't bother checking _which_ exact interrupt line fired - there's only
    // ever going to be one in this example.
    unsafe {
        let exti = &*stm32::EXTI::ptr();
        exti.pr.write(|w| w.pr10().set_bit());
    }

    let ps2 = unsafe { ps2::PS2::conjure() };
    let (data, valid) = read_reg!(ps2, ps2, RX, data, valid);
    if valid != 0 {
        rprintln!("key {:02x}", data);
    }
}