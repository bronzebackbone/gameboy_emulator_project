
#[derive(Debug)]
pub struct Timer {
    pub div: u16,
    pub div_mask: u16,
    pub tima: u8,
    pub tma: u8,
    pub is_tima_enable: bool,
    pub is_tima_overflowed: bool,


}

impl Timer {
    pub fn new() -> Self {
        Timer{
            div: 0,
            div_mask: 0x03FF,
            tima: 0,
            tma: 0,
            is_tima_enable: false,
            is_tima_overflowed: false,
        }
    }
    pub fn tick(&mut self, iff: &mut u8) {
        if self.is_tima_enable && ((self.div & self.div_mask) == self.div_mask) {
            if self.is_tima_overflowed {
                self.is_tima_overflowed = false;
                self.tima = self.tma;
                *iff |= 1 << 2;
            }else {
                self.tima = self.tima.wrapping_add(1);
                if self.tima == 0 {
                    self.is_tima_overflowed = true;
                }
            }
        }

        self.div = self.div.wrapping_add(1);
    }

    pub fn read_div(&self) -> u8 {
        (self.div >> 8) as u8
    }

    pub fn write_tac(&mut self, val: u8) {
        self.is_tima_enable = (val & 0x04) != 0;
        self.div_mask = match val & 0x03 {
            0x0 => 0x03FF,
            0x1 => 0x000F,
            0x2 => 0x003F,
            0x3 => 0x00FF,
            _ => unreachable!(),
        };
    }

    pub fn read_tac(&self) -> u8 {
        let enable = (self.is_tima_enable as u8) << 2;
        let freq: u8 = match self.div_mask {
            0x03FF => 0x0,
            0x000F => 0x1,
            0x003F => 0x2,
            0x00FF => 0x3,
            _ => panic!("Timer TAC bitmask at unintended value: {:04X}", self.div_mask),
        };
        
        0xF8 | enable | freq
    }
}