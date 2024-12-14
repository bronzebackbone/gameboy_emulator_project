use std::collections::VecDeque;
use crate::ppu::pixel::Pixel;
use crate::ppu::oam::Obj;
use crate::bus::Bus;

#[derive(Debug)]
pub struct Fetcher{
    pub state: FetcherState,
    // pub oldstate: FetcherState,
    pub dots: u8,
    pub fifo: VecDeque<Pixel>,

    pub mapaddr: u16,
    pub xoffset: u8,
    pub tiledataaddr: u16,
    pub tileline: u8,
    pub tileid: u8,
    pub tiledata0: u8,
    pub tiledata1: u8,
    pub tileidsigned: bool,

    pub obj: Obj,
    pub objoffset: u8,
    pub objtileline: u8,
    pub objflags: u8,
    pub objoamindex: u8,

    pub divider: u8,

    pub is_disabled: bool,
}

#[derive(Copy,Clone, PartialEq, Debug)]
pub enum FetcherState {
    ReadTileId,
    ReadTileData0,
    ReadTileData1,
    PushToFifo,
    ReadSpriteId,
    ReadSpriteFlags,
    ReadSpriteData0,
    ReadSpriteData1,
    MixInFifo,
}

impl Fetcher {
    pub fn new() -> Self {
        Fetcher{
            state: FetcherState::ReadTileId,
            // oldstate: FetcherState::ReadTileId,
            dots: 0,
            fifo: VecDeque::new(),

            mapaddr: 0,
            xoffset: 0,
            tiledataaddr: 0,
            tileline: 0,
            tileidsigned: false,
            tileid: 0,
            tiledata0: 0,
            tiledata1: 0,

            obj: Obj::default(),
            objoffset: 0,
            objtileline: 0,
            objflags: 0,
            objoamindex: 0,

            divider: 0,

            is_disabled: false,

        }
    }
    pub fn reset(&mut self) {
        self.state = FetcherState::ReadTileId;
        self.tileid = 0;
        self.tiledata0 = 0;
        self.tiledata1 = 0;
        self.divider = 0;
        self.is_disabled = false;
    }
    pub fn tick(&mut self, bus: &mut Bus) {
        if self.is_disabled && (self.state == FetcherState::ReadTileId) {
            if self.fifo.len() <= 8 {
                self.fifo.append(&mut [Pixel::default(); 8].into());
            }
            return;
        }
        
        self.divider += 1;
        if self.divider < 2 {
            return;
        }else {
            self.divider = 0;
        }

        match self.state {
            FetcherState::ReadTileId => {
                self.tileid = bus.ppuread(self.mapaddr.wrapping_add(self.xoffset as u16));
                self.state = FetcherState::ReadTileData0;
            }
            FetcherState::ReadTileData0 => {
                self.tiledata0 = self.get_bgwintile_data(bus, 0);
                self.state = FetcherState::ReadTileData1;
            }
            FetcherState::ReadTileData1 => {
                self.tiledata1 = self.get_bgwintile_data(bus, 1);
                self.state = FetcherState::PushToFifo;
            }
            FetcherState::PushToFifo => {
                if self.fifo.len() <= 8 {
                    self.fifo.append(&mut Pixel::zip(self.tiledata0, self.tiledata1, false, None, None).into());
                    self.xoffset = (self.xoffset + 1) & 0x1F;
                    self.state = FetcherState::ReadTileId;
                }
            }
            FetcherState::ReadSpriteId => {
                self.tileid = bus.ppuread(self.obj.oamaddr + 2);
                self.state = FetcherState::ReadSpriteFlags;
            }
            FetcherState::ReadSpriteFlags => {
                self.objflags = bus.ppuread(self.obj.oamaddr + 3);
                self.state = FetcherState::ReadSpriteData0;
            }
            FetcherState::ReadSpriteData0 => {
                if (bus.lcdc & 0x04) != 0 {
                    self.tileid &= 0xFE;
                }
                self.tiledata0 = self.get_objtile_data(bus, 0);
                self.state = FetcherState::ReadSpriteData1;

            }
            FetcherState::ReadSpriteData1 => {
                self.tiledata1 = self.get_objtile_data(bus, 1);
                self.state = FetcherState::MixInFifo;
            }
            FetcherState::MixInFifo => {
                let palette = Some((self.objflags & 0x10) != 0);
                let bgpriority = Some((self.objflags & 0x80) != 0);
                let reverse = (self.objflags & 0x20) != 0;
                let pixelline = Pixel::zip(self.tiledata0, self.tiledata1, reverse, palette, bgpriority);
                for (j, pixel) in pixelline.iter().enumerate().skip(self.objoffset as usize) {
                    let i = j - (self.objoffset as usize);
                    if let Some(fifopixel) = self.fifo.get_mut(i) {
                        if fifopixel.bgpriority.is_some() {
                            continue;
                        }
                        if (bgpriority.unwrap() && (fifopixel.color == 0)) || ((!bgpriority.unwrap()) && (pixel.color != 0)) {
                            *fifopixel = *pixel;
                        }
                    }else {
                        self.fifo.push_back(*pixel);
                    }
                }
                // for j in (self.objoffset as usize)..=7 {
                //     let pixel = pixelline[j];
                //     let i = j - (self.objoffset as usize);
                //     if let Some(fifopixel) = self.fifo.get_mut(i) {
                //         if fifopixel.bgpriority.is_some() {
                //             continue;
                //         }

                //         if (bgpriority.unwrap() && (fifopixel.color == 0)) || ((!bgpriority.unwrap()) && (pixel.color != 0)) {
                //             *fifopixel = pixel;
                //         }
                //     }else {
                //         self.fifo.push_back(pixel);
                //     }
                // }
                self.state = FetcherState::ReadTileId;
            }
        }
    }
    pub fn start_fetch(&mut self, mapaddr: u16, tiledataaddr: u16, xoffset: u8, tileidsigned: bool, tileline: u8) {
        self.mapaddr = mapaddr;
        self.tiledataaddr = tiledataaddr;
        self.xoffset = xoffset;
        self.tileidsigned = tileidsigned;
        self.tileline = tileline;
        self.fifo.clear();

        self.state = FetcherState::ReadTileId;
        self.tileid = 0;
        self.tiledata0 = 0;
        self.tiledata1 = 0;
    }
    pub fn start_sprite_fetch(&mut self, bus: &mut Bus, obj: Obj, offset: u8, oamindex: u8) {
        self.obj = obj;
        // self.oldstate = self.state;
        self.state = FetcherState::ReadSpriteId;
        self.objtileline = bus.ly + 16 - obj.ypos;
        self.objoffset = offset;
        self.objoamindex = oamindex;
    }
    
    pub fn get_bgwintile_data(&self, bus: &mut Bus, bytenumber: u8) -> u8 {
        let tileaddr = if self.tileidsigned {
            self.tiledataaddr.wrapping_add((self.tileid as i8  as u16) << 4)
        }else {
            self.tiledataaddr.wrapping_add((self.tileid as u16) << 4)
        };
        bus.ppuread(tileaddr | ((self.tileline as u16) << 1) | (bytenumber as u16))
    }
    pub fn get_objtile_data(&self, bus: &mut Bus, bytenumber: u8) -> u8 {
        let tileheight = if (bus.lcdc & 0x04) != 0 {16u8} else {8u8}; 
        let effectiveline = if (self.objflags & 0x40) != 0 {
            tileheight - 1 - self.objtileline
        }else {
            self.objtileline
        };
        let tileaddr = 0x8000 | ((self.tileid as u16) << 4);
        bus.ppuread(tileaddr | ((effectiveline as u16) << 1) | (bytenumber as u16))
    }

    
}

pub fn pixelzip(data0: u8, data1: u8) -> [char; 8] {
    let mut char_line = ['0'; 8];
    let palette = ['\u{2588}','\u{2593}', '\u{2592}', '\u{2591}'];
    for i in 0..=7 {
        let mask = (1 << i) as u8;
        let color = ((((data1 & mask) != 0) as u8) << 1) | (((data0 & mask) != 0) as u8);
        char_line[7 - i] = if color != 0 {palette[3]} else {palette[0]};
    }
    char_line
}

pub fn display_tiles(bus: &mut Bus) {
    let iter = (&bus.vram[0x1800..=0x1FFF]).iter()
        .filter(|&&x| x != 0)
        .map(|&x| {
            let addr: usize = (x as usize) << 4;
            &bus.vram[addr..(addr+16)]
        })
        .collect::<Vec<_>>();
                
    for slice in iter {
        for i in 0..=7 {
            let c = pixelzip(slice[2*i], slice[(2*i)+1]);
            println!("{}{}{}{}{}{}{}{}", c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]);
        }
        println!();
    }
}