#[derive(Debug)]
pub struct LengthCounter {
    pub enabled: bool,
    pub max_length: u16,
    pub counter: u16,
}

impl LengthCounter {
    pub fn new(max_length: u16) -> Self {
        Self{
            enabled: false,
            max_length,
            counter: 0,
        }
    }
    pub fn tick(&mut self, channel_enable: &mut bool) {
        if self.enabled {

            self.counter = self.counter.saturating_sub(1);
            if self.counter == 0 {
                *channel_enable = false;
            }
        }
    }
    pub fn write_length(&mut self, val: u8) {
        self.counter = self.max_length.wrapping_sub(val as u16);
    }

    pub fn write_enable(&mut self, val: u8) {
        self.enabled = ((val >> 6) & 0x01) != 0;
    }

    pub fn read_nrx4(&self) -> u8{
        (0xBF)|((self.enabled as u8) << 6)
    }

    pub fn trigger(&mut self, length_next: bool,) {
        if self.counter == 0 {
            self.counter = self.max_length - (!length_next && self.enabled) as u16;
        }
    }
}