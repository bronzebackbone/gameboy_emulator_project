use crate::bus::Bus;
pub struct Dma {
    index: u8,
}

impl Dma {
    pub fn new() -> Self {
        Dma{
            index: 0,
        }
    }
    pub fn tick(&mut self, bus: &mut Bus, tstates: usize) {
        if (tstates % 4) != 0 {
            return;
        }

        if bus.is_oam_dma {
            let sourceaddr = ((bus.dma as u16) << 8) | (self.index as u16);
            let val = bus.oamdmaread(sourceaddr);
            bus.oamdmawrite(self.index, val);
            self.index += 1;
            if self.index >= 160 {
                self.index = 0;
                bus.is_oam_dma = false;
            }       
        }
    }
}