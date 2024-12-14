use crate::bus::Bus;

pub struct Oam {
    pub state: OamState,
    pub index: u8,
    pub curr_obj: Obj,
    pub obj_table: Vec<Obj>,

}
pub enum OamState {
    ReadObjY,
    ReadObjX,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Obj {
    pub ypos: u8,
    pub xpos: u8,
    pub oamaddr: u16,
    pub is_fetched: bool,
}
impl Oam {
    pub fn new() -> Self {
        Oam{
            state: OamState::ReadObjY,
            index: 0,
            curr_obj: Obj::default(),
            obj_table: Vec::new(),
        }
    }
    pub fn reset(&mut self) {
        self.state = OamState::ReadObjY;
        self.index = 0;
        self.curr_obj = Obj::default();
        self.obj_table.clear();
    }
    
    pub fn tick(&mut self, bus: &mut Bus) -> bool {
        self.curr_obj.oamaddr = 0xFE00 | ((self.index as u16) << 2);
        match self.state {
            OamState::ReadObjY => {
                self.curr_obj.ypos = bus.ppuread(self.curr_obj.oamaddr);
                self.state = OamState::ReadObjX;
            }
            OamState::ReadObjX => {
                if self.obj_table.len() == 10 {
                    return true
                }
                let mut height = 8u8;
                if bus.lcdc & 0x04 != 0 {
                    height = 16;
                }
                self.curr_obj.xpos = bus.ppuread(self.curr_obj.oamaddr + 1);
                if self.curr_obj.xpos != 0 {
                    let y = bus.ly + 16;
                    if self.curr_obj.ypos <= y && (self.curr_obj.ypos + height) > y {
                        self.obj_table.push(self.curr_obj);
                    }
                }
                self.state = OamState::ReadObjY;
                self.index += 1;
            }
        }
        return self.index >= 40
    }
}