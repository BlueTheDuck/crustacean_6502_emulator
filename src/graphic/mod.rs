pub struct Image {
    pub pixels: std::boxed::Box<[Color]>,
    pub width: usize,
    pub height: usize,
}
#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl Image {
    pub fn new() -> Self {
        let width: usize = 16;
        let height: usize = 16;
        let pixels: Vec<_> = (0..(width * height)).map(|_| Color::from(0i32)).collect();
        let pixels = pixels.into_boxed_slice();
        Self {
            pixels,
            width,
            height,
        }
    }
    pub fn draw(&self, cr: &cairo::Context) {
        println!("Started drawing");
        cr.scale(self.width as f64, self.height as f64);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.paint();
        for y in 0..self.height {
            for x in 0..self.width {
                let (r, g, b): (f64, _, _) = self
                    .pixels
                    .get(y * self.width + x)
                    .expect("Out of bounds")
                    .into();
                cr.set_source_rgb(r, g, b);
                cr.rectangle(
                    x as f64 / self.width as f64,
                    y as f64 / self.height as f64,
                    1.,
                    1.,
                );
                cr.fill();
            }
        }
    }
}
impl std::convert::From<(u8, u8, u8)> for Color {
    fn from(color: (u8, u8, u8)) -> Self {
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
            r: (num >> 16) as u8,
            g: (num >> 8) as u8,
            b: (num) as u8,
        }
    }
}
impl std::convert::Into<(f64, f64, f64)> for &Color {
    fn into(self) -> (f64, f64, f64) {
        (
            (self.r as f64) / 255.0,
            (self.g as f64) / 255.0,
            (self.b as f64) / 255.0,
        )
    }
}
