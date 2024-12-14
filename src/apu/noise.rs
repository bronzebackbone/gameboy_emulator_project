use super::{envelope::Envelope, lengthcounter::LengthCounter};

#[derive(Debug)]
pub struct Noise{
    pub left_enable: bool,
    pub right_enable: bool,
    pub enabled: bool,

    pub envelope: Envelope,
    pub length_counter: LengthCounter,

    pub shift_clock_freq: u8,
    pub width_mode: bool,
    pub divisor_code: u8,

    pub freq_timer: u16,
    pub lsfr: u16,

    pub dac_enable: bool,

    pub dac_capacitor: f32,
}
impl Default for Noise{
    fn default() -> Self {
        Self{
            left_enable: false,
            right_enable: false,
            enabled: false,
            envelope: Envelope::default(),
            length_counter: LengthCounter::new(64),
            shift_clock_freq: 0,
            width_mode: false,
            divisor_code: 0,
            freq_timer: 0,
            lsfr: 0,
            dac_enable: false,
            dac_capacitor: 0.0,
        }
    }
}
impl Noise {
    pub fn tick(&mut self) {
        self.freq_timer = self.freq_timer.saturating_sub(1);

        if self.freq_timer == 0 {
            self.step_lsfr();
            self.freq_timer = self.base_divisor() << self.shift_clock_freq;
        }
    }


    pub fn base_divisor(&self) -> u16 {
        if self.divisor_code == 0 {
            8
        }else {
            self.divisor_code as u16 * 16
        }
    }

    pub fn step_lsfr(&mut self) {
        let xor: u16 = (self.lsfr & 1) ^ ((self.lsfr >> 1) & 1);
        self.lsfr >>= 1;

        self.lsfr |= xor << 14;
        
        if self.width_mode {
            self.lsfr &= !0x40;
            self.lsfr |= xor << 6;
        }
    }

    pub fn output(&self) -> u8 {
        if !self.enabled {
            0
        }else {
            ((self.lsfr & 1) ^ 1) as u8 * self.envelope.current_vol
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

    pub fn read_nr43(&self) -> u8{
        ((self.shift_clock_freq & 0x0F) << 4)|((self.width_mode as u8) << 3)|(self.divisor_code & 0x07)
    }

    pub fn write_nr42(&mut self, val: u8) {
        self.envelope.write_envelope(val);
        self.dac_enable = (val & 0xF8) != 0;
        if !self.dac_enable {
            self.enabled = false;
        } 
    }

    pub fn write_nr43(&mut self, val: u8) {
        self.shift_clock_freq = val >> 4;
        self.width_mode = (val & 0x80) != 0;
        self.divisor_code = val & 0x07;
    }

    pub fn write_nr44(&mut self, length_next: bool, val: u8) {
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
        if self.dac_enable{
            self.enabled = true;
        }
        self.length_counter.trigger(length_next);
        self.envelope.trigger();
        self.lsfr = 0x7FFF;
    }
}

/*
noise freq =

CPU_FREQ 
--------------
(16 * divcode kinda) << shiftfreq

example
divisor: 
001
=> loaded into freq divider:
110


*/