use super::envelope::Envelope;
use super::lengthcounter::LengthCounter;

#[derive(Debug)]
pub struct Square{
    pub left_enable: bool,
    pub right_enable: bool,
    pub enabled: bool,

    pub envelope: Envelope,
    pub length_counter: LengthCounter,


    pub sweep_period: u8,
    pub sweep_negate: bool,
    pub sweep_shift: u8,

    pub freq_shadow: u16,
    pub sweep_enable: bool,
    pub sweep_timer: u8,

    pub freq: u16,
    pub freq_timer: u16,
    pub duty: u8,
    pub phase: u8,

    pub dac_enable: bool,

    pub dac_capacitor: f32,
}

const DUTY_CYCLES: [[u8; 8]; 4] = [
    [0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,1],
    [1,0,0,0,0,1,1,1],
    [0,1,1,1,1,1,1,0],
];
impl Default for Square{
    fn default() -> Self {
        Self{
            left_enable: false,
            right_enable: false,
            enabled: false,
            envelope: Envelope::default(),
            length_counter: LengthCounter::new(64),
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            freq_shadow: 0,
            sweep_enable: false,
            sweep_timer: 0,
            freq: 0,
            freq_timer: 0,
            duty: 0,
            phase: 0,
            dac_enable: false,
            dac_capacitor: 0.0,
        }
    }
}
impl Square {
    pub fn tick(&mut self) {
        self.freq_timer = self.freq_timer.saturating_sub(1);

        if self.freq_timer == 0 {
            self.phase = (self.phase + 1) % 8;
            self.freq_timer = 4*(2048 - self.freq);
        }
    }
    
    pub fn tick_sweeper(&mut self) {
        self.sweep_timer = self.sweep_timer.saturating_sub(1);
        if self.sweep_timer == 0 {
            self.sweep_timer = if self.sweep_period != 0 {self.sweep_period} else {8};
            
            if self.sweep_enable && self.sweep_period != 0 {
                let new_freq = self.sweep_calculation();
                if new_freq <= 2047 && self.sweep_shift != 0 {
                    self.freq = new_freq;
                    self.freq_shadow = new_freq;
                    self.sweep_calculation();
                }
            }
        }
    }

    pub fn sweep_calculation(&mut self) -> u16 {
        let shifted_freq = self.freq_shadow >> self.sweep_shift;
        let new_freq = if self.sweep_negate {
            self.freq_shadow.wrapping_sub(shifted_freq)
        }else {
            self.freq_shadow.wrapping_add(shifted_freq)
        };
        if new_freq > 2047 {
            self.enabled = false;
        }

        new_freq
    }

    pub fn output(&self) -> u8 {
        if !self.enabled {
            0
        }else {
            DUTY_CYCLES[self.duty as usize & 0x03][self.phase as usize & 0x07] * self.envelope.current_vol
        }
    }

    pub fn dac_output(&mut self) -> f32 {
        let output = self.output();
        // if output == 0{
        //     0.0
        // }else {
        //     let dac_in = output as f32 / 15.0;
        //     let dac_out = dac_in - self.dac_capacitor;
        //     self.dac_capacitor = dac_in - dac_out * 0.992;
        //     dac_out
        // }

        // (output as f32 - 7.5)/7.5
        if (!self.enabled) || (self.envelope.current_vol == 0) {
            0.0
        }else {
            let dac_in = output as f32 / 15.0;
            let dac_out = dac_in - self.dac_capacitor;
            self.dac_capacitor = dac_in - dac_out * 0.992;
            dac_out
        }
    }

    pub fn read_nr10(&self) -> u8 {
        (0x80)|((self.sweep_period & 0x07) << 4)|((self.sweep_negate as u8) << 3)|(self.sweep_shift & 0x07)
    }
    pub fn read_nrx1(&self) -> u8 {
        (0x3F)|((self.duty & 0x03) << 6)
    }

    pub fn write_nr10(&mut self, val: u8) {
        self.sweep_period = (val >> 4) & 0x07;
        self.sweep_negate = (val & 0x08) != 0;
        self.sweep_shift = val & 0x07;
    }

    pub fn write_nrx1(&mut self, val: u8, power: bool) {
        if power {
            self.duty = val >> 6;
        }
        self.length_counter.write_length(val);
    }

    pub fn write_nrx2(&mut self, val: u8) {
        self.envelope.write_envelope(val);
        self.dac_enable = (val & 0xF8) != 0;
        if !self.dac_enable {
            self.enabled = false;
        } 
    }

    pub fn write_nrx3(&mut self, val: u8) {
        self.freq = (self.freq & 0xFF00)|(val as u16); 
    }

    pub fn write_nrx4(&mut self, length_next: bool, val: u8) {
        self.freq = (self.freq & 0x00FF)|(((val & 0x07) as u16) << 8);
        let old_length_enable = self.length_counter.enabled;
        self.length_counter.write_length(val);
        let new_length_enable = self.length_counter.enabled;

        if !length_next && !old_length_enable && new_length_enable {
            self.length_counter.tick(&mut self.enabled);
        }
        if (val & 0x80) != 0 {
            self.trigger(length_next);
        }
    }

    pub fn trigger(&mut self, length_next: bool) {
        if self.dac_enable {
            self.enabled = true;
        }
        self.freq_timer = 4*(2048 - self.freq);
        self.length_counter.trigger(length_next);
        self.envelope.trigger();
        self.sweep_trigger();
    }

    pub fn sweep_trigger(&mut self) {
        self.freq_shadow = self.freq;
        self.sweep_timer = if self.sweep_period != 0 {self.sweep_period} else {8};
        self.sweep_enable = (self.sweep_period != 0) || (self.sweep_shift != 0);

        if self.sweep_shift != 0 {
            self.sweep_calculation();
        }
    }
    
}