pub mod square;
pub mod wave;
pub mod noise;
pub mod envelope;
pub mod lengthcounter;


use self::square::Square;
use self::wave::Wave;
use self::noise::Noise;

#[derive(Debug)]
pub struct Apu{
    pub lvol: u8,  // S02 output level 0-7
    pub rvol: u8, // S01 output level 0-7
    pub lvin: bool,
    pub rvin: bool,
    pub enable: bool,

    pub ch1: Square,
    pub ch2: Square,
    pub ch3: Wave,
    pub ch4: Noise,

    pub sequencer_step: i8, 

    pub div_bit: bool,

    pub sample_counter: f64,

    pub buffer: Vec<f32>,

    pub dbgch1: bool,
    pub dbgch2: bool,
    pub dbgch3: bool,
    pub dbgch4: bool, 
}

impl Apu {
    pub fn new() -> Self {
        Self{
            lvol: 0,
            rvol: 0,
            lvin: false,
            rvin: false,
            enable: false,
            ch1: Square::default(),
            ch2: Square::default(),
            ch3: Wave::default(),
            ch4: Noise::default(),
            sequencer_step: 0,
            div_bit: false,
            sample_counter: 0.0,
            buffer: Vec::new(),
            dbgch1: true,
            dbgch2: true,
            dbgch3: true,
            dbgch4: true,
        }
    }

    pub fn power_off(&mut self) {
        for i in 0xFF10..=0xFF25 {
            self.writeu8(i, 0);
        }
        self.ch1.enabled = false;
        self.ch2.enabled = false;
        self.ch3.enabled = false;
        self.ch4.enabled = false;
    }

    pub fn power_on(&mut self) {
        self.sequencer_step = 0;
        if self.div_bit {
            self.sequencer_step = -1;
        }
        self.ch1.phase = 0;
        self.ch2.phase = 0;
        self.ch3.table_index = 0;
    }
    
    pub fn tick(&mut self, divider: u8) {
        const SAMPLE_RATE: f64 = 22_050_f64;
        const SAMPLE_EVERY_N_TICKS: f64 = 4_194_304_f64 / SAMPLE_RATE;

        self.sample_counter += 1.0;
        if self.sample_counter >= SAMPLE_EVERY_N_TICKS {
            self.push_output();
            self.sample_counter -= SAMPLE_EVERY_N_TICKS;
        }

        if !self.enable {
            return;
        }

        self.ch1.tick();
        self.ch2.tick();
        self.ch3.tick();
        self.ch4.tick();

        let old_div_bit = self.div_bit;
        self.div_bit = (divider >> 4) & 1 != 0;

        if old_div_bit && !self.div_bit {
            match self.sequencer_step{

                0 | 2 | 4 | 6 => {

                    self.ch1.length_counter.tick(&mut self.ch1.enabled);
                    self.ch2.length_counter.tick(&mut self.ch2.enabled);
                    self.ch3.length_counter.tick(&mut self.ch3.enabled);
                    self.ch4.length_counter.tick(&mut self.ch4.enabled);

                    if let 2 | 6 = self.sequencer_step {
                        self.ch1.tick_sweeper();
                    }
                }

                7 => {
                    self.ch1.envelope.tick();
                    self.ch2.envelope.tick();
                    self.ch4.envelope.tick();
                }

                1 | 3 | 5 => (),

                -1 => (),

                err => panic!("{}", err),
            }
            self.sequencer_step = if self.sequencer_step < 7{
                self.sequencer_step + 1
            }else {
                0
            };
        }
    }

    pub fn push_output(&mut self) {
        let rvol = self.rvol as f32 + 1.0;
        let lvol = self.lvol as f32 + 1.0;

        let ch1 = if self.dbgch1 {self.ch1.dac_output() / 8.0}else {0.0};
        let ch2 = if self.dbgch2 {self.ch2.dac_output() / 8.0}else {0.0};
        let ch3 = if self.dbgch3 {self.ch3.dac_output() / 8.0}else {0.0};
        let ch4 = if self.dbgch4 {self.ch4.dac_output() / 8.0}else {0.0};

        let ch1_right = if self.ch1.right_enable {ch1 * rvol}else {0.0};
        let ch2_right = if self.ch2.right_enable {ch2 * rvol}else {0.0};
        let ch3_right = if self.ch3.right_enable {ch3 * rvol}else {0.0};
        let ch4_right = if self.ch4.right_enable {ch4 * rvol}else {0.0};

        let ch1_left = if self.ch1.left_enable {ch1 * lvol}else {0.0};
        let ch2_left = if self.ch2.left_enable {ch2 * lvol}else {0.0};
        let ch3_left = if self.ch3.left_enable {ch3 * lvol}else {0.0};
        let ch4_left = if self.ch4.left_enable {ch4 * lvol}else {0.0};

        let right_sample = ch1_right + ch2_right + ch3_right + ch4_right;
        let left_sample = ch1_left + ch2_left + ch3_left + ch4_left;
        
        self.buffer.push(left_sample);
        self.buffer.push(right_sample);
    }
    
    pub fn readu8(&self, addr: u16) -> u8{
        match addr{
            0xFF10 => self.ch1.read_nr10(),
            0xFF11 => self.ch1.read_nrx1(),
            0xFF12 => self.ch1.envelope.read_nrx2(),
            0xFF13 => 0xFF,
            0xFF14 => self.ch1.length_counter.read_nrx4(),

            0xFF15 => 0xFF,
            0xFF16 => self.ch2.read_nrx1(),
            0xFF17 => self.ch2.envelope.read_nrx2(),
            0xFF18 => 0xFF,
            0xFF19 => self.ch2.length_counter.read_nrx4(),

            0xFF1A => self.ch3.read_nr30(),
            0xFF1B => 0xFF,
            0xFF1C => self.ch3.read_nr32(),
            0xFF1D => 0xFF,
            0xFF1E => self.ch3.length_counter.read_nrx4(),
            
            0xFF1F => 0xFF,
            0xFF20 => 0xFF,
            0xFF21 => self.ch4.envelope.read_nrx2(),
            0xFF22 => self.ch4.read_nr43(),
            0xFF23 => self.ch4.length_counter.read_nrx4(),

            0xFF24 => self.read_nr50(),
            0xFF25 => self.read_nr51(),
            0xFF26 => self.read_nr52(),

            0xFF27..=0xFF2F => 0xFF,

            0xFF30..=0xFF3F => self.ch3.read_wave_ram((addr & 0x000F) as usize),
            _ => unreachable!(),
        }
    }

    pub fn writeu8(&mut self, addr: u16, val: u8) {
        if !self.enable && addr <= 0xFF25 && (addr % 5 != 2 || addr == 0xFF25) {
            return;
        }
        let length_next = self.is_length_clock_next();
        match addr{
            0xFF10 => self.ch1.write_nr10(val),
            0xFF11 => self.ch1.write_nrx1(val, self.enable),
            0xFF12 => self.ch1.write_nrx2(val),
            0xFF13 => self.ch1.write_nrx3(val),
            0xFF14 => self.ch1.write_nrx4(length_next, val),

            0xFF15 => (),
            0xFF16 => self.ch2.write_nrx1(val, self.enable),
            0xFF17 => self.ch2.write_nrx2(val),
            0xFF18 => self.ch2.write_nrx3(val),
            0xFF19 => self.ch2.write_nrx4(length_next, val),

            0xFF1A => self.ch3.write_nr30(val),
            0xFF1B => self.ch3.length_counter.write_length(val),
            0xFF1C => self.ch3.write_nr32(val),
            0xFF1D => self.ch3.write_nr33(val),
            0xFF1E => self.ch3.write_nr34(length_next, val),

            0xFF1F => (),
            0xFF20 => self.ch4.length_counter.write_length(val),
            0xFF21 => self.ch4.write_nr42(val),
            0xFF22 => self.ch4.write_nr43(val),
            0xFF23 => self.ch4.write_nr44(length_next, val),

            0xFF24 => self.write_nr50(val),
            0xFF25 => self.write_nr51(val),
            0xFF26 => self.write_nr52(val),
            
            0xFF27..=0xFF2F => (),
            0xFF30..=0xFF3F => self.ch3.write_wave_ram((addr & 0x000F) as usize, val),
            _ => unreachable!(),
        }
    }

    pub fn read_nr50(&self) -> u8 {
        ((self.lvin as u8) << 7)|((self.lvol & 0x07) << 4)|((self.rvin as u8) << 3)|(self.rvol & 0x07)
    }
    pub fn read_nr51(&self) -> u8 {
        ((self.ch4.left_enable as u8) << 7) |
        ((self.ch3.left_enable as u8) << 6) |
        ((self.ch2.left_enable as u8) << 5) |
        ((self.ch1.left_enable as u8) << 4) |
        ((self.ch4.right_enable as u8) << 3)|
        ((self.ch3.right_enable as u8) << 2)|
        ((self.ch2.right_enable as u8) << 1)|
        (self.ch1.right_enable as u8)
    }

    pub fn read_nr52(&self) -> u8 {
        (0x70)|((self.enable as u8) << 7)
        |((self.ch4.enabled as u8) << 3)
        |((self.ch3.enabled as u8) << 2)
        |((self.ch2.enabled as u8) << 1)
        |(self.ch1.enabled as u8)
    }

    pub fn write_nr50(&mut self, val: u8) {
        self.lvin = (val & 0x80) != 0;
        self.lvol = (val >> 4) & 0x07;
        self.rvin = (val & 0x08) != 0;
        self.rvol = val & 0x07;
    }

    pub fn write_nr51(&mut self, val: u8) {
        self.ch4.left_enable = (val & 0x80) != 0;
        self.ch3.left_enable = (val & 0x40) != 0;
        self.ch2.left_enable = (val & 0x20) != 0;
        self.ch1.left_enable = (val & 0x10) != 0;
        self.ch4.right_enable = (val & 0x08) != 0;
        self.ch3.right_enable = (val & 0x04) != 0;
        self.ch2.right_enable = (val & 0x02) != 0;
        self.ch1.right_enable = (val & 0x01) != 0;
    }
    
    pub fn write_nr52(&mut self, val: u8) {
        let new_enable = (val & 0x80) != 0;
        if self.enable && !new_enable {
            self.power_off();
        }else if !self.enable && new_enable{
            self.power_on();
        }

        self.enable = new_enable;
    }

    pub fn is_length_clock_next(&self) -> bool {
        (self.sequencer_step % 2) == 0
    }
}