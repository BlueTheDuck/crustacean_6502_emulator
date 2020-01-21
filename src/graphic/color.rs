#[derive(Copy, Clone, Debug)]
pub struct Color {
    r: f64,
    g: f64,
    b: f64,
}
impl std::convert::From<(u8, u8, u8)> for Color {
    fn from(color: (u8, u8, u8)) -> Self {
        Color {
            r: (color.0 as f64) / 255.0,
            g: (color.1 as f64) / 255.0,
            b: (color.2 as f64) / 255.0,
        }
    }
}
impl std::convert::From<(f64, f64, f64)> for Color {
    fn from(color: (f64, f64, f64)) -> Self {
        Color {
            r: color.0,
            g: color.1,
            b: color.2,
        }
    }
}
impl std::convert::From<i32> for Color {
    fn from(num: i32) -> Self {
        let num = num & 0x00_FF_FF_FF; // Remove alpha
        Color {
            r: (num >> 16) as f64 / 255.0,
            g: (num >> 8) as f64 / 255.0,
            b: (num) as f64 / 255.0,
        }
    }
}
impl std::convert::Into<(f64, f64, f64)> for &Color {
    fn into(self) -> (f64, f64, f64) {
        (self.r, self.g, self.b)
    }
}
impl std::default::Default for Color {
    fn default() -> Self {
        Color::from(0x00_00_00)
    }
}
