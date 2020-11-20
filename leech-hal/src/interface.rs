use cortex_m::interrupt::free as interrupt_free;
use litex_pac::register::MemoryInterface;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::gpio::{Input, Floating, Output, PushPull, Alternate, AF0, AF5, Speed};
use stm32f4xx_hal::gpio::gpioa::PA8;
use stm32f4xx_hal::gpio::gpiob::{PB2, PB10, PB12, PB13, PB14, PB15};
use stm32f4xx_hal::spi::Spi;
use stm32f4xx_hal::hal::spi::MODE_0;
use stm32f4xx_hal::stm32::SPI2;

pub struct SpiMemoryInterface {
    spi: Spi<SPI2, (PB13<Alternate<AF5>>, PB14<Alternate<AF5>>, PB15<Alternate<AF5>>)>,
    cs: PB12<Output<PushPull>>,
    creset: PB2<Output<PushPull>>,
    _mco: PA8<Alternate<AF0>>,
    _irq: PB10<Input<Floating>>,
    us_ticks: u32,
}

impl SpiMemoryInterface {
    pub fn init<CS, SCK, MISO, MOSI, CRESET, MCO, IRQ>(
        spi: SPI2,
        cs: PB12<CS>,
        sck: PB13<SCK>,
        miso: PB14<MISO>,
        mosi: PB15<MOSI>,
        creset: PB2<CRESET>,
        mco: PA8<MCO>,
        irq: PB10<IRQ>,
        clocks: Clocks,
    ) -> Self {
        let mut creset = creset.into_push_pull_output();
        creset.set_low().ok();

        let mut cs = cs.into_push_pull_output();
        cs.set_high().ok();
        let sck = sck.into_alternate_af5().set_speed(Speed::VeryHigh);
        let miso = miso.into_alternate_af5().set_speed(Speed::VeryHigh);
        let mosi = mosi.into_alternate_af5().set_speed(Speed::VeryHigh);
        let spi = Spi::spi2(spi, (sck, miso, mosi), MODE_0, 8_000_000.hz(), clocks);

        // Setup MCO
        unsafe {
            // 16MHz output
            let rcc = &*stm32f4xx_hal::stm32::RCC::ptr();
            rcc.cfgr.modify(|_, w| {
                w.mco1().hsi();
                w.mco1pre().div1()
            });
        }

        let us_ticks = clocks.sysclk().0 / 1_000_000;

        Self {
            spi,
            cs,
            creset,
            _mco: mco.into_alternate_af0(),
            _irq: irq.into_floating_input(),
            us_ticks,
        }
    }

    pub fn upload_bitstream(&mut self, bitstream: &[u8]) {
        self.creset.set_low().ok();
        self.cs.set_low().ok();
        self.delay_us(10); // >=200ns
        self.creset.set_high().ok();

        self.delay_us(1500); // >=1200us

        self.cs.set_high().ok();
        self.spi.write(&[0]).unwrap();
        self.cs.set_low().ok();

        self.spi.write(bitstream).unwrap();
        self.spi.write(&[0; 6]).unwrap();

        self.cs.set_high().ok();
    }

    pub fn install(&mut self) {
        unsafe {
            let ptr = self as *mut _;
            crate::pac::register::set_memory_interface(&mut *ptr);
        }
    }

    fn delay_us(&mut self, us: u32) {
        cortex_m::asm::delay(self.us_ticks * us);
    }
}

impl MemoryInterface for SpiMemoryInterface {
    fn read32(&mut self, address: u32) -> u32 {
        let address = (address >> 2).to_le_bytes();
        let mut buffer = [0x03, address[0], address[1], 0x00, 0xcc, 0xcc, 0xcc, 0xcc];

        interrupt_free(|_| {
            self.cs.set_low().ok();
            self.delay_us(1);

            self.spi.transfer(&mut buffer).unwrap();

            self.delay_us(1);
            self.cs.set_high().ok();
            self.delay_us(1);
        });

        u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]])
    }

    fn write32(&mut self, address: u32, value: u32) {
        let address = (address >> 2).to_le_bytes();
        let value = value.to_le_bytes();
        let mut buffer = [0x02, address[0], address[1], value[0], value[1], value[2], value[3]];

        interrupt_free(|_| {
            self.cs.set_low().ok();
            self.delay_us(1);

            self.spi.transfer(&mut buffer).unwrap();

            self.delay_us(1);
            self.cs.set_high().ok();

            self.delay_us(1);
        });
    }
}
