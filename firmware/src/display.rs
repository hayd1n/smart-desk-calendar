use epd_waveshare::{
    color::Color,
    epd7in5_v2::{HEIGHT, WIDTH},
    graphics::{VarDisplay, VarDisplayError},
};

pub use epd_waveshare::color::Color::{Black as White, White as Black};

pub const DISPLAY_BUFFER_SIZE: usize = epd_waveshare::buffer_len(WIDTH as usize, HEIGHT as usize);

pub fn create_display() -> Result<VarDisplay<'static, Color>, VarDisplayError> {
    let display_buffer = vec![0; DISPLAY_BUFFER_SIZE].into_boxed_slice();
    VarDisplay::<Color>::new(WIDTH, HEIGHT, Box::leak(display_buffer), false)
}
