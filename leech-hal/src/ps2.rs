use crate::pac::peripherals::ps2;
use crate::pac::{read_reg, write_reg};

pub struct PS2 {
    ps2: ps2::Instance,
}

impl PS2 {
    pub fn new(ps2: ps2::Instance) -> PS2 {
        PS2 { ps2 }
    }

    pub fn listen(&mut self) {
        write_reg!(ps2, self.ps2, EV_ENABLE, 1);
    }

    pub fn unlisten(&mut self) {
        write_reg!(ps2, self.ps2, EV_ENABLE, 0);
    }

    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let mut count = 0;
        for b in buf.iter_mut() {
            let (data, valid) = read_reg!(ps2, self.ps2, RX, data, valid);
            if valid != 0 {
                *b = data as u8;
                count += 1;
            } else {
                break;
            }
        }
        count
    }
}
