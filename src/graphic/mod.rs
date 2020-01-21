mod color;
mod palette;
pub use color::Color;

pub struct Image {
    pub pixels: std::boxed::Box<[Color]>,
    pub width: usize,
    pub height: usize,
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
    pub fn draw(&self, cr: &cairo::Context, widget_size: (i32, i32)) {
        //println!("Started drawing");
        /* println!("Info: ");
        println!("Image {}x{}", w, h);
        println!("Widget {}x{}", widget_width, widget_height);
        println!("Pixel {}x{}", pixel_width, pixel_height); */
        let w = self.width as f64;
        let h = self.height as f64;
        let widget_width = widget_size.0 as f64;
        let widget_height = widget_size.1 as f64;
        let pixel_width = 1. / w;
        let pixel_height = 1. / h;
        cr.scale(widget_width, widget_height);
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
                    x as f64 * pixel_width,
                    y as f64 * pixel_height,
                    pixel_width,
                    pixel_height,
                );
                cr.fill();
            }
        }
    }
}
