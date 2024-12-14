use super::lengthcounter::LengthCounter;

#[derive(Debug)]
pub struct Wave{
    pub left_enable: bool,
    pub right_enable: bool,
    pub enabled: bool,

    pub length_counter: LengthCounter,

    pub freq: u16,
    pub freq_timer: u16,

    pub vol: u8,

    pub wave_table: [u8; 16],
    pub table_index: u8,

    pub dac_enable: bool,

    pub dac_capacitor: f32,
}

const VOL_SHIFT: [u8; 4] = [4, 0, 1, 2];

impl Default for Wave{
    fn default() -> Self {
        Self{
            left_enable: false,
            right_enable: false,
            enabled: false,
            length_counter: LengthCounter::new(256),
            freq: 0,
            freq_timer: 0,
            vol: 0,
            wave_table: [0; 16],
            table_index: 0,
            dac_enable: false,
            dac_capacitor: 0.0,
        }
    }
}
impl Wave {
    pub fn new() -> Self {
        todo!()
    }

    pub fn tick(&mut self) {
        self.freq_timer = self.freq_timer.saturating_sub(1);
        if self.freq_timer == 0 {
            self.freq_timer = 2*(2048 - self.freq);
            self.table_index = (self.table_index + 1) & 0x1F;

        }
    }
    
    pub fn output(&self) -> u8 {
        if !self.enabled{
            0
        }else {
            let byte = self.wave_table[(self.table_index as usize) >> 1];
            let shift = 4*((self.table_index & 1) ^ 1);
            let sample = (byte >> shift) & 0x0F;
            
            sample >> VOL_SHIFT[self.vol as usize & 0x03]
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
        if !self.enabled{
            0.0
        }else {
            let dac_in = output as f32 / 15.0;
            let dac_out = dac_in - self.dac_capacitor;
            self.dac_capacitor = dac_in - dac_out * 0.992;
            dac_out
        }
    }

    pub fn read_nr30(&self) -> u8{
        (0x7F)|((self.dac_enable as u8) << 7)
    }
    
    pub fn read_nr32(&self) -> u8{
        (0x9F)|((self.vol & 0x03) << 5)
    }

    pub fn write_nr30(&mut self, val: u8) {
        self.dac_enable = (val & 0x80) != 0;
        if !self.dac_enable {
            self.enabled = false;
        }
    }

    pub fn write_nr32(&mut self, val: u8) {
        self.vol = (val >> 5) & 0x03;
    }

    pub fn write_nr33(&mut self, val: u8) {
        self.freq = (self.freq & 0xFF00)|(val as u16);
    }

    pub fn write_nr34(&mut self, length_next: bool, val: u8) {
        self.freq = (self.freq & 0x00FF)|(((val & 0x07) as u16) << 8);
        let old_length_enable = self.length_counter.enabled;
        self.length_counter.write_enable(val);
        let new_length_enable = self.length_counter.enabled;
        if !length_next && !old_length_enable && new_length_enable {
            self.length_counter.tick(&mut self.enabled);
        }
        if (val & 0x80) != 0 {
            self.trigger(length_next);
        }
    }

    pub fn read_wave_ram(&self, offset: usize) -> u8 {
        let index = if self.dac_enable && self.enabled {
            (self.table_index >> 1) as usize
        }else {
            offset
        } & 0x0F;
        self.wave_table[index]
    }

    pub fn write_wave_ram(&mut self, offset: usize, val: u8) {
        let index = if self.dac_enable && self.enabled {
            (self.table_index >> 1) as usize
        }else {
            offset
        } & 0x0F;
        self.wave_table[index] = val;
    }

    pub fn trigger(&mut self, length_next: bool) {
        if self.dac_enable {
            self.enabled = true;
        }
        self.length_counter.trigger(length_next);
        self.table_index = 0;
        self.freq_timer = 2*((2048 - self.freq) + 2);
    }
}