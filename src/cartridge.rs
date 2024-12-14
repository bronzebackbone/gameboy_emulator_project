use std::time::SystemTime;
#[derive(Debug)]
pub enum Mbc{
    RomOnly,
    Mbc1{
        bank_mode: bool,
        is_ram_enable: bool,
        rom_bank_lo: u8,
        rom_bank_hi: u8,
    },
    Mbc2{
        is_ram_enable: bool,
        rom_bank: u8,
    },
    Mbc3{
        is_enable: bool,
        rtcs: u8,
        rtcm: u8,
        rtch: u8,
        rtcdl: u8,
        rtcdh: u8,
        rom_bank: u8,
        ram_or_rtc: u8,
        latched: u8,
    },
    Mbc5{
        is_ram_enable: bool,
        rom_bank_lo: u8,
        rom_bank_hi: u8,
        ram_bank: u8,
    }
}
#[derive(Debug)]
pub struct Cartridge {
    pub rom: Vec<u8>,
    pub bootrom: Vec<u8>,
    pub sram: Vec<u8>,
    pub romsize: usize,
    pub rombank: usize,
    pub ramsize: usize,
    pub rambank: usize,
    pub mbc: Mbc,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>, bootrom: Vec<u8>) -> Self {
        let romsize = 32768 << rom[0x0148];
        let rombank = romsize / 16384;
        let ramsize = match rom[0x0149] {
            0x00 | 0x01 => 0,
            0x02 => 8192,
            0x03 => 32768,
            0x04 => 131072,
            0x05 => 65536,
            _ => panic!("ERROR: Unknown ram size at cartridge initialization"),
        };
        let rambank = ramsize / 8192;
        let mbc = match rom[0x0147] {
            0x00 => Mbc::RomOnly,
            0x01..=0x03 => Mbc::Mbc1{bank_mode: false, is_ram_enable: false, rom_bank_lo: 0x01, rom_bank_hi: 0x00},
            0x05 | 0x06 => Mbc::Mbc2{is_ram_enable: false, rom_bank: 0x01},
            0x0F..=0x13 => Mbc::Mbc3{
                is_enable: false,
                rtcs: get_rtc_s(),
                rtcm: get_rtc_m(),
                rtch: get_rtc_h(),
                rtcdh: get_rtc_dh(),
                rtcdl: get_rtc_dl(),
                rom_bank: 0x01,
                ram_or_rtc: 0x00,
                latched: 0xFF,
            },
            0x19..=0x1E => Mbc::Mbc5{is_ram_enable: false, rom_bank_hi: 0x00, rom_bank_lo: 0x00, ram_bank: 0x00},
            _ => panic!("Unknown / Unsupported MBC at cartridge initialization"),
        };
        if let Mbc::Mbc2{is_ram_enable: _, rom_bank: _} = mbc {
            let _ramsize = 512;
        }
        let sram = vec![0u8; ramsize];
        Cartridge{
            rom,
            bootrom,
            romsize,
            rombank,
            ramsize,
            rambank,
            sram,
            mbc,
        }
    }
    
    pub fn readu8(&mut self, addr: u16) -> u8 {
        // println!("Cart read => {:04x}", addr);
        match self.mbc {
            Mbc::RomOnly => {
                match addr {
                    0x0000..=0x7FFF => self.rom[addr as usize],
                    0xA000..=0xBFFF => self.sram[(addr & 0x1FFF) as usize],
                    _ => panic!("Should be unreachable cartridge read reached at {:x}", addr),
                }
            }
            Mbc::Mbc1{bank_mode, is_ram_enable, rom_bank_lo, rom_bank_hi} => {
                match addr {
                    0x0000..=0x3FFF => {
                        if !bank_mode || self.romsize < 524288{
                            self.rom[addr as usize]
                        }else {
                            self.rom[((rom_bank_hi as usize & 0x03) << 19) | addr as usize]
                        }
                    }
                    0x4000..=0x7FFF => {
                        self.rom[
                            ((rom_bank_hi as usize & 0x03) << 19)
                            |((rom_bank_lo as usize & 0x1F) << 14)
                            |(addr as usize & 0x3FFF)
                            & !(self.romsize)
                            ]
                    } 
                    0xA000..=0xBFFF if !is_ram_enable => 0xFF,
                    0xA000..=0xBFFF if self.romsize <= 524_288 => {
                        if !bank_mode {
                            self.sram[(addr & 0x1FFF) as usize]
                        } else {
                            self.sram[((rom_bank_hi as usize & 0x03) << 13) | (addr & 0x1FFF) as usize]
                        }
                    }
                    0xA000..=0xBFFF => self.sram[(addr & 0x1FFF) as usize],
                    _ => panic!("Should be unreachable cartridge read reached at {:x}", addr),
                }
            }
            Mbc::Mbc2{is_ram_enable, rom_bank} => {
                match addr {
                    0x0000..=0x3FFF => self.rom[addr as usize],
                    0x4000..=0x7FFF => self.rom[((rom_bank as usize & 0x0F) << 14)|(addr as usize & 0x3FFF)],
                    0xA000..=0xBFFF if !is_ram_enable => 0xFF,
                    0xA000..=0xBFFF => self.sram[addr as usize & 0x01FF] & 0x0F,
                    _ => panic!("Should be unreachable cartridge read reached at {:x}", addr),
                }
            }
            Mbc::Mbc3{is_enable,rtcs, rtcm, rtch, rtcdl, rtcdh, rom_bank, ram_or_rtc, latched:_} => {
                match addr {
                    0x0000..=0x3FFF => self.rom[addr as usize],
                    0x4000..=0x7FFF => {
                        // println!("Cart read at {:6X}", ((rom_bank as usize & 0x7F) << 14)|(addr as usize & 0x3FFF));
                        self.rom[((rom_bank as usize & 0x7F) << 14)|(addr as usize & 0x3FFF)]
                    }
                    0xA000..=0xBFFF if !is_enable => 0xFF, 
                    0xA000..=0xBFFF => {
                        match ram_or_rtc {
                            0x00..=0x03 => self.sram[((ram_or_rtc as usize & 0x03) << 13)|(addr as usize & 0x1FFF)],
                            0x08 => rtcs,
                            0x09 => rtcm,
                            0x0A => rtch,
                            0x0B => rtcdl,
                            0x0C => rtcdh,
                            _ => panic!("Read at undefined Mbc2 ram bank number {:x}", ram_or_rtc)
                        }
                    }
                    _ => panic!("Should be unreachable cartridge write reached at {:x}", addr),
                }
            }
            Mbc::Mbc5{is_ram_enable, rom_bank_hi, rom_bank_lo, ram_bank} => {
                match addr {
                    0x0000..=0x3FFF => self.rom[addr as usize],
                    0x4000..=0x7FFF => {
                        self.rom[
                            ((rom_bank_hi as usize & 0x01) << 22)
                            |((rom_bank_lo as usize) << 14)
                            |(addr as usize & 0x3FFF)
                            ]
                    }
                    0xA000..=0xBFFF if !is_ram_enable => 0xFF,
                    0xA000..=0xBFFF => self.sram[((ram_bank as usize & 0x0F) << 13)|(addr as usize & 0x1FFF)],
                    _ => panic!("Should be unreachable cartridge read reached at {:x}", addr),
                }
            }

        }
    }

    pub fn writeu8(&mut self, addr: u16, val: u8) {
        match &mut self.mbc {
            Mbc::RomOnly => {
                match addr {
                    0x0000..=0x7FFF => (),
                    0xA000..=0xBFFF => self.sram[(addr & 0x1FFF) as usize] = val,
                    _ => panic!("Should be unreachable cartridge write reached at {:x}", addr),
                }
            }
            Mbc::Mbc1{bank_mode, is_ram_enable, rom_bank_lo, rom_bank_hi} => {
                match addr{
                    0x0000..=0x1FFF if val & 0x0F == 0x0A => *is_ram_enable = true,
                    0x0000..=0x1FFF => *is_ram_enable = false,
                    0x2000..=0x3FFF if val == 0x00 => *rom_bank_lo = 0x01,
                    0x2000..=0x3FFF => *rom_bank_lo = val & 0x1F,
                    0x4000..=0x5FFF => *rom_bank_hi = val & 0x03,
                    0x6000..=0x7FFF => *bank_mode = val != 0,
                    0xA000..=0xBFFF if !*is_ram_enable => (),
                    0xA000..=0xBFFF if self.romsize <= 524_288 => {
                        if !*bank_mode {
                            self.sram[(addr & 0x1FFF) as usize] = val;
                        }else {
                            self.sram[((*rom_bank_hi as usize & 0x03) << 13)| (addr & 0x1FFF) as usize] = val;
                        }
                    }
                    0xA000..=0xBFFF => self.sram[(addr & 0x1FFF) as usize] = val,
                    _ => panic!("Should be unreachable cartridge write reached at {:x}", addr),
                }
            }
            Mbc::Mbc2{is_ram_enable, rom_bank} => {
                match addr {
                    0x0000..=0x3FFF if addr & 0x0100 == 0 => if val != 0x0A {*is_ram_enable = false;} else {*is_ram_enable = true;},
                    0x0000..=0x3FFF if val == 0x00 => *rom_bank = 0x01,
                    0x0000..=0x3FFF => *rom_bank = val & 0x0F,
                    0xA000..=0xBFFF if !*is_ram_enable => (),
                    0xA000..=0xBFFF => self.sram[addr as usize & 0x01FF] = val & 0x0F,
                    _ => panic!("Should be unreachable cartridge write reached at {:x}", addr),
                }
            }
            Mbc::Mbc3{is_enable, rtcs, rtcm, rtch, rtcdl, rtcdh, rom_bank, ram_or_rtc, latched} => {
                match addr {
                    0x0000..=0x1FFF if val == 0x0A => *is_enable = true,
                    0x0000..=0x1FFF => *is_enable = false,
                    0x2000..=0x3FFF if val == 0x00 => *rom_bank = 0x01,
                    0x2000..=0x3FFF => {
                        // println!("Rom bank changed to {}", val & 0x7F);
                        *rom_bank = val & 0x7F;
                    }
                    0x4000..=0x5FFF => *ram_or_rtc = val & 0x0F,
                    0x6000..=0x7FFF if *latched == 0x00 && val == 0x01 => {
                        *latched = 0x01;
                        *rtcs = get_rtc_s();
                        *rtcm = get_rtc_m();
                        *rtch = get_rtc_h();
                        *rtcdl = get_rtc_dl();
                        *rtcdh = get_rtc_dh();
                    }
                    0x6000..=0x7FFF => *latched = val,
                    0xA000..=0xBFFF if !*is_enable => (),
                    0xA000..=0xBFFF => {
                        match *ram_or_rtc {
                            0x00..=0x03 => self.sram[((*ram_or_rtc as usize & 0x03) << 13)|(addr as usize & 0x1FFF)] = val,
                            0x08 => *rtcs = val,
                            0x09 => *rtcm = val,
                            0x0A => *rtch = val,
                            0x0B => *rtcdl = val,
                            0x0C => *rtcdh = val,
                            _ => panic!("Write at undefined Mbc2 ram bank {:x}", *ram_or_rtc),
                        }
                    } 
                    _ => panic!("Should be unreachable cartridge write reached at {:x}", addr),
                }
            }
            Mbc::Mbc5{is_ram_enable, rom_bank_hi, rom_bank_lo, ram_bank} => {
                match addr {
                    0x0000..=0x1FFF if val == 0x0A => *is_ram_enable = true,
                    0x0000..=0x1FFF => *is_ram_enable = false,
                    0x2000..=0x2FFF => *rom_bank_lo = val,
                    0x3000..=0x3FFF => *rom_bank_hi = val & 0x01,
                    0x4000..=0x5FFF => *ram_bank = val & 0x0F,
                    0x6000..=0x7FFF => (),
                    0xA000..=0xBFFF if !*is_ram_enable => (),
                    0xA000..=0xBFFF => self.sram[((*ram_bank as usize & 0x0F) << 13)|(addr as usize & 0x1FFF)] = val,
                    _ => panic!("Should be unreachable cartridge write reached at {:x}", addr),
                }
            }
        }
    }
    pub fn read_bootrom(&mut self, addr: u16) -> u8 {
        self.bootrom[addr as usize & 0xFF]
    }
}
pub fn secs_since_epoch() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}
pub fn get_rtc_s() -> u8 {
    (secs_since_epoch() % 60) as u8
}
pub fn get_rtc_m() -> u8 {
    ((secs_since_epoch() / 60) % 60) as u8
}
pub fn get_rtc_h() -> u8 {
    ((secs_since_epoch() / 3600) % 24) as u8
}
pub fn get_rtc_dl() -> u8 {
    (((secs_since_epoch() / 86400) as f64 % 365.2425) as u16 & 0x00FF) as u8 
}
pub fn get_rtc_dh() -> u8 {
    ((((secs_since_epoch() / 86400) as f64 % 365.2425) as u16 & 0x0100) >> 8) as u8
}