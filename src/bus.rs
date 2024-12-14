use crate::cartridge::Cartridge;
use super::timer::Timer;
use super::apu::Apu;

#[derive(Debug)]
pub struct Bus{
    pub cart: Cartridge,     //0000-3FFF fixed bank
                             //4000-7FFF switchable banks
    
    pub timer: Timer, 

    pub apu: Apu,

    pub vram:  [u8; 0x2000], //8000-9FFF
                             
                             //A000-BFFF sram, from cartridge switchable if any

    pub wram0: [u8; 0x1000], //C000-CFFF
    pub wramn: [u8; 0x1000], //D000-DFFF
                             //E000-FDFF Mirrors C000-DDFF
    pub oam:   [u8; 0x00A0], //FE00-FE9F
                             //FEA0-FEFF Unusable memory
    pub p1:   u8,            //FF00 Joypad IO reg
    pub sb:   u8,            //FF01 Serial transfer data R/W
    pub sc:   u8,            //FF02 Serial transfer control R/W
    // pub div:  u8,            //FF04 Divider register R/W
    // pub tima: u8,            //FF05 Timer counter R/W
    // pub tma:  u8,            //FF06 Timer Modulo R/W
    // pub tac:  u8,            //FF07 Timer control R/W

    pub iff:   u8,            //FF0F Interrupt flag R/W

    // Sound registers
    
   
    
    pub lcdc: u8,
    pub stat: u8,
    pub scy:  u8,
    pub scx:  u8,
    pub ly:   u8,
    pub lyc:  u8,
    pub dma:  u8,
    pub wy:   u8,
    pub wx:   u8,
    pub bgp:  u8,
    pub obp0: u8,
    pub obp1: u8,
    pub hram: [u8; 0x007F], //FF80-FFFE
    pub ie: u8, //FFFF

    pub ime: bool,
    pub imebuf: bool,
    pub is_cpu_halt : bool,
    pub is_boot_rom: bool,
    pub is_oam_dma: bool,
    pub is_ppu_mode23: bool,
    pub is_ppu_mode3: bool,
    pub is_vram_block: bool,


    pub jpad_down: bool,
    pub jpad_up: bool,
    pub jpad_right: bool,
    pub jpad_left: bool,
    pub jpad_a: bool,
    pub jpad_b: bool,
    pub jpad_select: bool,
    pub jpad_start: bool,

    pub debug_inst: bool,
}

impl Bus {
    pub fn new(cart: Cartridge) -> Bus {

        Bus{
            cart,
            timer: Timer::new(),
            vram: [0; 0x2000],
            wram0: [0; 0x1000],
            wramn: [0; 0x1000],
            oam: [0; 0x00A0],
            p1: 0x0F,
            sb: 0x00,
            sc: 0x00,
            
            iff: 0x00,
            
            apu: Apu::new(),
            
            lcdc: 0x00,
            stat: 0x00,
            scy: 0x00,
            scx: 0x00,
            ly: 0x00,
            lyc: 0x00,
            dma: 0x00,
            wy: 0x00,
            wx: 0x00,
            bgp: 0x00,
            obp0: 0x00,
            obp1: 0x00,
            hram: [0; 0x007F],
            ie: 0x00,
            ime: false,
            imebuf: false,
            is_cpu_halt: false,
            is_boot_rom: true,
            is_oam_dma: false,
            is_ppu_mode23: true,
            is_ppu_mode3: false,
            is_vram_block: false,


            jpad_down: false,
            jpad_up: false,
            jpad_right: false,
            jpad_left: false,
            jpad_a: false,
            jpad_b: false,
            jpad_select: false,
            jpad_start: false,

            debug_inst: false,
        }
    }
    pub fn after_bootup(&mut self) {
        self.p1 = 0xCF;
        self.sb = 0x00;
        self.sc = 0x7E;
        self.timer.div = 0xAB00;
        // self.div = 0xAB;
        // self.tima = 0x00;
        // self.tma = 0x00;
        // self.tac = 0xF8;
        self.iff = 0xE1;
        
        self.lcdc = 0x91;
        self.stat = 0x85;
        self.scy = 0x00;
        self.scx = 0x00;
        self.ly = 0x00;
        self.lyc = 0x00;
        self.bgp = 0xFC;
        self.wy = 0x00;
        self.wx = 0x00;
        self.ie = 0x00;
    }
    pub fn readu8(&mut self, addr: u16) -> u8 {
        if !self.is_oam_dma {
            match addr {
                0x0000..=0x00FF if self.is_boot_rom => self.cart.read_bootrom(addr),
                0x0000..=0x7FFF => self.cart.readu8(addr),
                0x8000..=0x9FFF if !self.is_ppu_mode3 => self.vram[(addr & 0x1FFF) as usize],
                0x8000..=0x9FFF => 0xFF,
                0xA000..=0xBFFF => self.cart.readu8(addr),
                0xC000..=0xCFFF | 0xE000..=0xEFFF => self.wram0[(addr & 0x0FFF) as usize],
                0xD000..=0xDFFF | 0xF000..=0xFDFF => self.wramn[(addr & 0x0FFF) as usize],
                0xFE00..=0xFE9F if !self.is_ppu_mode23 => self.oam[(addr & 0x00FF) as usize],
                0xFE00..=0xFE9F => 0xFF,
                0xFEA0..=0xFEFF => 0xFF,
                0xFF00 if (self.p1 & 0x20) == 0 => self.read_act(),
                0xFF00 if (self.p1 & 0x10) == 0 => self.read_dir(),
                0xFF00 => 0xCF,
                0xFF01 => self.sb,
                0xFF02 => self.sc,
                0xFF03 => 0xFF,
                0xFF04 => self.timer.read_div(),
                0xFF05 => self.timer.tima,
                0xFF06 => self.timer.tma,
                0xFF07 => self.timer.read_tac(),
                0xFF08..=0xFF0F => 0xFF,
                
                0xFF10..=0xFF3F => self.apu.readu8(addr),

                0xFF40 => self.lcdc,
                0xFF41 => self.stat,
                0xFF42 => self.scy,
                0xFF43 => self.scx,
                0xFF44 => self.ly,
                0xFF45 => self.lyc,
                0xFF46 => self.dma,
                0xFF47 => self.bgp,
                0xFF48 => self.obp0,
                0xFF49 => self.obp1,
                0xFF4A => self.wy,
                0xFF4B => self.wx,
                0xFF4C..=0xFF4F => 0xFF,
                0xFF50 => self.is_boot_rom as u8,
                0xFF51..=0xFF7F => 0xFF,
                0xFF80..=0xFFFE => self.hram[(addr & 0x007F) as usize],
                0xFFFF => self.ie,

            }
        } else {
            match addr {
                0xFF80..=0xFFFE => self.hram[(addr & 0x007F) as usize],
                _ => 0xFF,                
            }

        }
    }

    pub fn writeu8(&mut self, addr: u16, val: u8) {
        if !self.is_oam_dma {
            match addr{
                0x0000..=0x7FFF => self.cart.writeu8(addr, val),
                0x8000..=0x9FFF if !self.is_ppu_mode3 => self.vram[(addr & 0x1FFF) as usize] = val,
                0x8000..=0x9FFF => (),
                0xA000..=0xBFFF => self.cart.writeu8(addr, val),
                0xC000..=0xCFFF | 0xE000..=0xEFFF => self.wram0[(addr & 0x0FFF) as usize] = val,
                0xD000..=0xDFFF | 0xF000..=0xFDFF => self.wramn[(addr & 0x0FFF) as usize] = val,
                0xFE00..=0xFE9F if !self.is_ppu_mode23 => self.oam[(addr & 0x00FF) as usize] = val,
                0xFE00..=0xFE9F => (),
                0xFEA0..=0xFEFF => (),
                0xFF00 => self.p1 = (val & 0x30)|(self.p1 & 0xCF),
                0xFF01 => self.sb = val,
                0xFF02 => self.sc = val,
                0xFF03 => (),
                0xFF04 => self.timer.div = 0,
                0xFF05 => self.timer.tima = val,
                0xFF06 => self.timer.tma = val,
                0xFF07 => self.timer.write_tac(val),
                0xFF08..=0xFF0F => (),

                0xFF10..=0xFF3F => self.apu.writeu8(addr, val),

                0xFF40 => self.lcdc = val,
                0xFF41 => self.stat = val & 0xF8,
                0xFF42 => self.scy = val,
                0xFF43 => self.scx = val,
                0xFF44 => (),
                0xFF45 => self.lyc = val,
                0xFF46 => self.start_oam_dma(val),
                0xFF47 => self.bgp = val,
                0xFF48 => self.obp0 = val,
                0xFF49 => self.obp1 = val,
                0xFF4A => self.wy = val,
                0xFF4B => self.wx = val,
                0xFF4C..=0xFF4F => (),
                0xFF50 => self.is_boot_rom = val == 0,
                0xFF51..=0xFF7F => (),
                0xFF80..=0xFFFE => self.hram[(addr & 0x007F) as usize] = val,
                0xFFFF => self.ie = val,
            }
        }else {
            match addr {
                0xFF80..=0xFFFE => self.hram[(addr & 0x007F) as usize] = val,
                _ => (),
            }
        }
    }
    pub fn ppuread(&mut self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF if self.is_vram_block => 0xFF,
            0x8000..=0x9FFF => self.vram[addr as usize & 0x1FFF],
            0xFE00..=0xFE9F if self.is_oam_dma => 0xFF,
            0xFE00..=0xFE9F => self.oam[addr as usize & 0x00FF],
            _ => panic!("The ppu read from an address it shouldn't! => {:04x}", addr), 
        }
    }

    pub fn oamdmaread(&mut self, addr: u16) -> u8 {
        let dmastate = self.is_oam_dma;
        self.is_oam_dma = false;
        let result = self.readu8(addr);
        self.is_oam_dma = dmastate;
        result
    }
    pub fn oamdmawrite(&mut self, addr: u8, val: u8) {
        self.oam[addr as usize] = val;
    }
    pub fn start_oam_dma(&mut self, src: u8) {
        self.is_oam_dma = true;
        self.dma = src;
    }

    pub fn read_dir(&self) -> u8 {
        let pressed = 0xE0u8;
        let dir = (((!self.jpad_down) as u8) << 3)|(((!self.jpad_up) as u8) << 2)|(((!self.jpad_left) as u8) << 1)|((!self.jpad_right) as u8);
        pressed|dir
    }
    pub fn read_act(&self) -> u8 {
        let pressed = 0xD0u8;
        let act = (((!self.jpad_start) as u8) << 3)|(((!self.jpad_select) as u8) << 2)|(((!self.jpad_b) as u8) << 1)|((!self.jpad_a) as u8);
        pressed|act
    }
    
} 
