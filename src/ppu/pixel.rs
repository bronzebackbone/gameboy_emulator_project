#[derive(Copy,Clone, Debug)]
pub struct Pixel {
    pub color: u8,
    pub palette: Option<bool>,
    pub bgpriority: Option<bool>,
}
impl Pixel {
    pub fn from_bg(color: u8) -> Self {
        Pixel {
            color: color & 0x03,
            palette: None,
            bgpriority: None,
        }
    }
    pub fn bg_disabled() -> Self {
        Pixel {
            color: 0,
            palette: None,
            bgpriority: None,
        }
    }
    pub fn zip(lo: u8, hi: u8, reverse: bool, palette: Option<bool>, bgpriority: Option<bool>) -> [Self; 8] {
        let mut pixel_line = [Pixel::default(); 8];
        let mut mask: u8;
        let mut pixel: Pixel;
        for i in 0..=7 {
            mask = (1 << i) as u8;
            pixel = Pixel{
                color: ((((hi & mask) != 0 ) as u8) << 1) | (((lo & mask) != 0) as u8),
                palette,
                bgpriority,
            };
            if reverse {
                pixel_line[i] = pixel;
            }else {
                pixel_line[7-i] = pixel;
            }
        }
        pixel_line
    }
}
impl Default for Pixel {
    fn default() -> Self {
        Pixel{
            color: 0,
            palette: None,
            bgpriority: None,
        }
    }
}