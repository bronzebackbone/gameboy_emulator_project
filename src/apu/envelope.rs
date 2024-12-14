#[derive(Debug, Default)]
pub struct Envelope{
    pub initial_vol: u8,
    pub current_vol: u8,
    pub sweep_increase: bool,
    pub period: u8,
    pub enabled: bool,
    pub counter: u8,
}

impl Envelope {
    pub fn tick(&mut self) {
        self.counter = self.counter.saturating_sub(1);
        if self.counter == 0{
            self.counter = if self.period != 0 {self.period} else {8};
            if self.enabled && self.period != 0 {
                if self.sweep_increase {
                    self.current_vol += if self.current_vol < 15 {1} else {0};
                }else {
                    self.current_vol = self.current_vol.saturating_sub(1);
                }

                if let 0 | 15 = self.current_vol {
                    self.enabled = false;
                }
            }
        }
    }
    pub fn read_nrx2(&self) -> u8{
        ((self.initial_vol & 0x0F) << 4)|((self.sweep_increase as u8) << 3)|(self.period & 0x07)
    }

    pub fn write_envelope(&mut self, val: u8) {
        self.initial_vol = val >> 4;
        self.current_vol = self.initial_vol;
        self.sweep_increase = (val & 0x08) != 0;
        self.period = val & 0x07;
        self.counter = self.period;
    }

    pub fn trigger(&mut self) {
        self.current_vol = self.initial_vol;
        self.counter = self.period;
        self.enabled = true;
    }
}