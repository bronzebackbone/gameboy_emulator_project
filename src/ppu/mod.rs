pub mod fetcher;
pub mod pixel;
pub mod oam;

use crate::ppu::fetcher::Fetcher;
use crate::ppu::fetcher::FetcherState;
use crate::ppu::oam::Oam;
// use crate::ppu::pixel::Pixel;


use crate::bus::Bus;

pub const WHITE: u8 = 0xFF;
pub const LIGHT_GRAY: u8 = 0xA9;
pub const DARK_GRAY: u8 = 0x54;
pub const BLACK: u8 = 0x00;

pub struct Ppu {
    pub state: PpuState,
    pub dots: usize,
    pub xpos: u8,

    pub fetcher: Fetcher,
    pub oam: Oam,

    pub to_drop: u8,
    pub is_window: bool,
    pub framebuffer: [u8; 3*160*144],
    pub entered_vblank: bool,

}
pub enum PpuState {
    OamSearch,
    PixelTransfer,
    HBlank,
    VBlank,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu{
            state: PpuState::OamSearch,
            dots: 0,
            xpos: 0,

            fetcher: Fetcher::new(),
            oam: Oam::new(),

            
            to_drop: 0,
            is_window: false,

            framebuffer: [0; 3*160*144],
            entered_vblank: false,

        }
    }
    
    pub fn tick(&mut self, bus: &mut Bus) {
        if (bus.lcdc & 0x80) == 0 {
            return;
        }
        self.dots += 1;
        match self.state {
            PpuState::OamSearch => {

                if self.oam.tick(bus) {
                    self.oam.index = 0;
                    self.xpos = 0;
                    self.is_window = false;
                    self.to_drop = bus.scx & 0x07;

                    self.fetcher.reset();
                    if (bus.lcdc & 0x01) != 0{
                        let bgx = (bus.scx & 0xF8) >> 3;
                        // let y = (((bus.scy as u16) + (bus.ly as u16)) & 0xFF) as u8;
                        let y = bus.scy.wrapping_add(bus.ly); 
                        let bgy = (y as u16 & 0xF8) << 2;
                        let bgtileline = y & 0x07;
                        let bgtilemap: u16 = if (bus.lcdc & 0x08) != 0 {0x9C00} else {0x9800};
                        let bgtiledata: u16 = if (bus.lcdc & 0x10) != 0 {0x8000} else {0x9000};
                        let is_signed = (bus.lcdc & 0x10) == 0;
                        self.fetcher.start_fetch(bgtilemap.wrapping_add(bgy), bgtiledata, bgx, is_signed, bgtileline);
                    }else {
                        self.fetcher.is_disabled = true;
                    }

                    bus.is_ppu_mode23 = true;
                    bus.is_ppu_mode3 = true;
                    self.state = PpuState::PixelTransfer;
                } 
            }
            PpuState::PixelTransfer => {
                self.fetcher.tick(bus);
                

                if bus.lcdc & 0x01 != 0 {
                    if self.fetcher.fifo.len() <= 8 {
                        return;
                    }
                    if self.to_drop > 0 {
                        if self.fetcher.fifo.pop_front().is_some() {
                            self.to_drop -= 1;
                        }
                        return;
                    }
                    if !self.is_window && (bus.lcdc & 0x20 != 0) && (bus.ly >= bus.wy) && ((self.xpos == bus.wx.wrapping_sub(7)) || (self.xpos == 0 && bus.wx < 7)) {
                        self.is_window = true;
                        self.to_drop = 0;
                        let winx = ((self.xpos + 7 - bus.wx) & 0xF8) >> 3;
                        let y = bus.ly.wrapping_sub(bus.wy);
                        let winy = ((y as u16) & 0xF8) << 2;
                        let wintileline = y & 0x07;
                        let wintilemap: u16 = if (bus.lcdc & 0x40) != 0 {0x9C00} else {0x9800};
                        let wintiledata: u16 = if (bus.lcdc & 0x10) != 0 {0x8000} else {0x9000};
                        let is_signed = (bus.lcdc & 0x10) == 0;
                        self.fetcher.start_fetch(wintilemap.wrapping_add(winy), wintiledata, winx, is_signed, wintileline);
                        return; 
                    }
                   
                }
                
                if bus.lcdc & 0x02 != 0 {
                    if matches!(self.fetcher.state,
                        FetcherState::ReadSpriteId 
                        |FetcherState::ReadSpriteFlags 
                        |FetcherState::ReadSpriteData0
                        |FetcherState::ReadSpriteData1
                        |FetcherState::MixInFifo
                    ){
                        return;
                    }
                    for (index, obj) in (self.oam.obj_table).iter_mut().enumerate() {
                        if obj.is_fetched {
                            continue;
                        }


                        if (obj.xpos < 8) && (self.xpos == 0) {
                            self.fetcher.start_sprite_fetch(bus, *obj, 8 - obj.xpos, index as u8);
                            obj.is_fetched = true;
                            return;
                        }else if (obj.xpos - 8) == self.xpos {
                            self.fetcher.start_sprite_fetch(bus, *obj, 0, index as u8);
                            obj.is_fetched = true;
                            return;
                        }

                    }
                }
                if let Some(pixel) = self.fetcher.fifo.pop_front() {
                    
                    let index = pixel.color & 0x03;
                    let palette;
                    if (index == 0 && pixel.bgpriority.is_some()) || (pixel.bgpriority.is_none()) || (pixel.palette.is_none()) {
                        palette = bus.bgp;
                    }else if pixel.palette.unwrap() {
                        palette = bus.obp1;
                    }else {
                        palette = bus.obp0;
                    }
                    let colorid = (palette & (0x03 << (2*index))) >> (2*index); 
                    let color = match colorid & 0x03 {
                        0x00 => WHITE,
                        0x01 => LIGHT_GRAY,
                        0x02 => DARK_GRAY,
                        0x03 => BLACK,
                        _ => panic!("This will never be reached, trust me"),
                    };

                    self.framebuffer[(3 * ((160 * bus.ly as usize) + self.xpos as usize)) + 0] = color;
                    self.framebuffer[(3 * ((160 * bus.ly as usize) + self.xpos as usize)) + 1] = color;
                    self.framebuffer[(3 * ((160 * bus.ly as usize) + self.xpos as usize)) + 2] = color;
                    self.xpos += 1;
                }
                if self.xpos == 160 {
                    bus.stat = (bus.stat & 0xFC)|(0x00);
                    if (bus.stat & 0x08) != 0 {
                        bus.iff |= 1 << 1;
                    }

                    bus.is_ppu_mode23 = false;
                    bus.is_ppu_mode3 = false;
                    self.state = PpuState::HBlank;

                }
            }
            PpuState::HBlank => {
                if self.dots >= 456 {
                    self.dots = 0;
                    bus.ly += 1;
                    if bus.ly == bus.lyc {
                        bus.stat |= 1 << 2;
                        if (bus.stat & 0x40) != 0 {
                            bus.iff |= 1 << 1;
                        }
                    }else {
                        bus.stat &= !(1 << 2);
                    }
                    if bus.ly == 144 {
                        bus.stat = (bus.stat & 0xFC)|(0x01);
                        if (bus.stat & 0x10) != 0 {
                            bus.iff |= 1 << 1;
                        }
                        bus.iff |= 1;

                        self.entered_vblank = true;

                        bus.is_ppu_mode23 = false;
                        bus.is_ppu_mode3 = false;
                        self.state = PpuState::VBlank;
                    }else {
                        bus.stat = (bus.stat & 0xFC)|(0x02);
                        if (bus.stat & 0x20) != 0 {
                            bus.iff |= 1 << 1;
                        }

                        bus.is_ppu_mode23 = true;
                        bus.is_ppu_mode3 = false;

                        self.oam.reset();
                        self.state = PpuState::OamSearch;
                    }
                }
            }
            PpuState::VBlank => {

                if self.dots >= 456 {
                    self.dots = 0;
                    bus.ly += 1;
                    if bus.ly == bus.lyc {
                        bus.stat |= 1 << 2;
                        if (bus.stat & 0x40) != 0 {
                            bus.iff |= 1 << 1;
                        }
                    }else {
                        bus.stat &= !(1 << 2);
                    }
                    if bus.ly == 154 {
                        bus.ly = 0;
                        if bus.ly == bus.lyc {
                            bus.stat |= 1 << 2;
                            if (bus.stat & 0x40) != 0 {
                                bus.iff |= 1 << 1;
                            }
                        }else {
                            bus.stat &= !(1 << 2);
                        }
                        bus.stat = (bus.stat & 0xFC)|(0x02);
                        if (bus.stat & 0x20) != 0 {
                            bus.iff |= 1 << 1;
                        }

                        bus.is_ppu_mode23 = true;
                        bus.is_ppu_mode3 = false;

                        self.oam.reset();
                        self.state = PpuState::OamSearch;
                    }
                }
            }
        }
    }

    
}